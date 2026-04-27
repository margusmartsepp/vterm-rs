//! PTY → parser → optional viewer fan-out.
//!
//! One blocking OS thread per terminal pulls bytes off the PTY, feeds the `vt100`
//! parser, increments the line counter, replies to cursor-position queries (`\x1b[6n`),
//! appends to a per-terminal log file, and forwards a copy to a bounded mpsc channel.
//! When the user opted into a visible viewer window, an async task drains that channel
//! over a per-terminal named pipe.

use std::fs::OpenOptions;
use std::io::{BufWriter, Read, Write};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe;
use tokio::sync::mpsc;

use super::Inner;

/// Start the pump for a terminal. Spawns one OS thread (the pump) and one tokio task
/// (the viewer drain).
pub fn start(
    inner: Arc<Inner>,
    reader: Box<dyn Read + Send>,
    visible_viewer: bool,
) {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);

    let parser = Arc::clone(&inner.parser);
    let writer = Arc::clone(&inner.writer);
    let line_count = Arc::clone(&inner.line_count);
    let id = inner.id;

    std::thread::Builder::new()
        .name(format!("pty-pump-{id}"))
        .spawn(move || pump_loop(id, reader, parser, writer, line_count, tx))
        .expect("spawn pty-pump");

    if visible_viewer {
        tokio::spawn(viewer_loop(id, rx, Arc::clone(&inner.writer)));
    } else {
        // The pump uses a bounded channel; if no viewer drains it the pump would
        // block. Drop bytes on the floor when there's no viewer.
        tokio::spawn(async move {
            let mut rx = rx;
            while rx.recv().await.is_some() {}
        });
    }
}

fn pump_loop(
    id: u32,
    mut reader: Box<dyn Read + Send>,
    parser: Arc<Mutex<vt100::Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    line_count: Arc<std::sync::atomic::AtomicU32>,
    tx: mpsc::Sender<Vec<u8>>,
) {
    let mut log = OpenOptions::new()
        .create(true).append(true)
        .open(format!("vterm-rs_{id}.log"))
        .ok()
        .map(BufWriter::new);

    let mut buf = [0u8; 8192];
    while match reader.read(&mut buf) {
        Ok(0) => {
            tracing::debug!(id, "pump_loop: read 0 bytes (EOF)");
            false
        }
        Ok(n) => {
            let chunk = &buf[..n];
            line_count.fetch_add(
                chunk.iter().filter(|&&b| b == b'\n').count() as u32,
                Ordering::Relaxed,
            );

            // Feed the parser. Mutex held only for the duration of `.process()`.
            parser.lock().process(chunk);

            // Cursor-position-report emulation: \x1b[6n -> \x1b[<row>;<col>R
            if memchr_subseq(chunk, b"\x1b[6n") {
                let (row, col) = parser.lock().screen().cursor_position();
                let resp = format!("\x1b[{};{}R", row + 1, col + 1);
                let mut w = writer.lock();
                let _ = w.write_all(resp.as_bytes());
                let _ = w.flush();
            }

            if let Some(log) = log.as_mut() {
                let _ = log.write_all(chunk);
                let _ = log.flush();
            }

            // `blocking_send` parks this OS thread when the channel is full, which is
            // exactly the back-pressure we want.
            if tx.blocking_send(chunk.to_vec()).is_err() { 
                tracing::debug!(id, "pump_loop: blocking_send failed");
                false 
            } else {
                true
            }
        }
        Err(e) => {
            tracing::warn!(id, error = %e, "pump_loop: read error");
            false
        }
    } {}
    tracing::debug!(id, "pump_loop exiting");
}

/// Cheap byte-subsequence search. Avoids pulling in `memchr` for one site.
#[inline]
fn memchr_subseq(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

async fn viewer_loop(
    id: u32,
    mut rx: mpsc::Receiver<Vec<u8>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
) {
    let pipe_name = format!(r"\\.\pipe\vterm-rs-client-{id}");
    let server = match named_pipe::ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(id, error = %e, "viewer pipe create failed");
            return;
        }
    };

    if server.connect().await.is_err() {
        return;
    }
    let (mut client_reader, mut client_writer) = tokio::io::split(server);

    // PTY → viewer
    let to_viewer = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if client_writer.write_all(&data).await.is_err() { break; }
            let _ = client_writer.flush().await;
        }
    });

    // viewer → PTY (so the user typing in the viewer reaches the underlying shell)
    let mut buf = [0u8; 1024];
    while let Ok(n) = client_reader.read(&mut buf).await {
        if n == 0 { break; }
        let mut w = writer.lock();
        let _ = w.write_all(&buf[..n]);
        let _ = w.flush();
    }
    to_viewer.abort();
    tracing::debug!(id, "viewer_loop exiting");
}
