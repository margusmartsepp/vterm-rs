//! PTY → parser → optional viewer fan-out.
//!
//! One blocking OS thread per terminal pulls bytes off the PTY, feeds the `vt100`
//! parser, increments the line counter, replies to cursor-position queries (`\x1b[6n`),
//! appends to a per-terminal log file, and forwards a copy to a bounded mpsc channel.
//! When the user opted into a visible viewer window, an async task drains that channel
//! over a per-terminal named pipe.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(windows)]
use tokio::net::windows::named_pipe;

use tokio::sync::mpsc;

use super::Inner;

/// Start the pump for a terminal. Spawns one OS thread (the pump) and one tokio task
/// (the viewer drain).
pub fn start(inner: Arc<Inner>, reader: Box<dyn Read + Send>, visible_viewer: bool) {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(65536); // Large buffer to avoid deadlocks during slow viewer startup

    let id = inner.id;
    let inner_for_pump = Arc::clone(&inner);

    std::thread::Builder::new()
        .name(format!("pty-pump-{id}"))
        .spawn(move || pump_loop(inner_for_pump, reader, tx))
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

fn pump_loop(inner: Arc<Inner>, mut reader: Box<dyn Read + Send>, tx: mpsc::Sender<Vec<u8>>) {
    let id = inner.id;
    let parser = Arc::clone(&inner.parser);
    let writer = Arc::clone(&inner.writer);
    let line_count = Arc::clone(&inner.line_count);
    let notifier = inner.notifier.clone();

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("vterm-rs_{id}.log"))
        .ok();

    let mut buf = [0u8; 8192];
    while let Ok(n) = reader.read(&mut buf) {
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];

        // Process in a block to ensure mutexes are dropped before potential blocking I/O
        {
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

            // Notify listeners that the screen state has changed
            let _ = notifier.send(());

            // Sync to SHM if enabled
            if let Some(shm) = &inner.shm {
                let screen = parser.lock().screen().clone();
                let contents = screen.contents();

                // 1. Detect screen clear (heuristic: content size dropped significantly or is empty)
                if contents.trim().is_empty() {
                    shm.clear_bloom();
                }

                // 2. Tokenize and update Bloom filter (post-parser rendered content)
                // We hash alphanumeric tokens to avoid ANSI noise and punctuation junk
                for token in contents.split(|c: char| !c.is_alphanumeric()) {
                    if token.len() > 1 {
                        shm.insert_token(token);
                    }
                }

                // 3. Update screen buffer and sequence
                shm.write_screen(contents.as_bytes());
                shm.update_header();
            }

            if let Some(log) = log.as_mut() {
                let _ = log.write_all(chunk);
                let _ = log.flush();
            }

            // Also append to the in-memory history buffer
            let mut history = inner.full_history.lock();
            history.push_str(&String::from_utf8_lossy(chunk));
        }

        // Forward to viewer. If channel is full, we drop bytes (the viewer is too slow).
        let _ = tx.try_send(chunk.to_vec());
    }
    let _ = notifier.send(()); // Final notify on EOF
    tracing::debug!(id, "pty-pump thread exiting");
}

fn memchr_subseq(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[cfg(windows)]
async fn viewer_loop(
    id: u32,
    mut rx: mpsc::Receiver<Vec<u8>>,
    writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
) {
    let pipe_name = format!(r"\\.\pipe\vterm-rs-client-{id}");
    let server = match named_pipe::ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(id, error = %e, "pipe create failed");
            return;
        }
    };

    tracing::debug!(id, "viewer_loop waiting for connection on {pipe_name}");
    if let Err(e) = server.connect().await {
        tracing::error!(id, error = %e, "pipe connect failed");
        return;
    }

    let (mut client_reader, mut client_writer) = tokio::io::split(server);

    // Pump from PTY to viewer
    let to_viewer = tokio::spawn(async move {
        while let Some(chunk) = rx.recv().await {
            if client_writer.write_all(&chunk).await.is_err() {
                break;
            }
            let _ = client_writer.flush().await;
        }
    });

    // Pump from viewer to PTY
    let mut buf = [0u8; 1024];
    while let Ok(n) = client_reader.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let mut w = writer.lock();
        let _ = w.write_all(&buf[..n]);
        let _ = w.flush();
    }
    to_viewer.abort();
    tracing::debug!(id, "viewer_loop exiting");
}

#[cfg(not(windows))]
async fn viewer_loop(
    _id: u32,
    mut rx: mpsc::Receiver<Vec<u8>>,
    _writer: Arc<Mutex<Box<dyn Write + Send>>>,
) {
    // Just drain the channel so the pump doesn't block
    while rx.recv().await.is_some() {}
}
