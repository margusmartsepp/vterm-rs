//! PTY ownership and lifecycle.
//!
//! ```text
//!   spawn() ─► Terminal<Spawning> ─► wait_for_prompt ─► Terminal<Ready>
//! ```
//!
//! The type-state machinery in [`instance`] makes "read before prompt" a *compile-time*
//! error, not a runtime race. The pump in [`pump`] runs on a dedicated blocking thread
//! per terminal so PTY I/O never holds an async runtime worker.

mod instance;
mod prompt;
pub mod shm;
#[cfg(windows)]
mod pump;

use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use regex::Regex;

use crate::{Error, Result};
use crate::protocol::SpawnArgs;

pub use instance::{Inner, Terminal, state};

pub struct SpawnResult {
    pub terminal: Terminal<state::Ready>,
    pub spawn_ms: u64,
    pub ready_ms: u64,
}

/// Spawn a fresh `powershell.exe`-backed terminal and block until its prompt appears.
///
/// `default_visible` is the orchestrator-wide fallback; `args.visible` overrides if set.
/// `prompt_regex` decides when the shell is ready to receive the optional initial command.
pub async fn spawn(
    id: u32,
    args: SpawnArgs,
    default_visible: bool,
    prompt_regex: &Regex,
) -> Result<SpawnResult> {
    let start = Instant::now();
    let visible = args.visible.unwrap_or(default_visible);

    // ── open the PTY ────────────────────────────────────────────────────────────
    let rows = args.rows.unwrap_or(24);
    let cols = args.cols.unwrap_or(80);
    tracing::info!(id, rows, cols, "opening PTY");
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| Error::Pty(format!("openpty: {e}")))?;
    tracing::info!(id, "PTY opened");

    let writer = pair.master.take_writer().map_err(|e| Error::Pty(format!("take_writer: {e}")))?;
    let reader = pair.master.try_clone_reader().map_err(|e| Error::Pty(format!("clone_reader: {e}")))?;

    // ── spawn the shell ─────────────────────────────────────────────────────────
    let mut cmd = CommandBuilder::new("powershell.exe");
    cmd.args(["-NoLogo", "-NoProfile", "-NoExit", "-ExecutionPolicy", "Bypass", "-Command", "Remove-Module PSReadLine -ErrorAction SilentlyContinue"]);
    cmd.env("TERM", "xterm-256color");
    if let Some(env) = args.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }

    tracing::info!(id, "spawning powershell.exe");
    let child = pair.slave.spawn_command(cmd).map_err(|e| Error::Pty(format!("spawn: {e}")))?;
    let child_pid = child.process_id().unwrap_or(0);
    tracing::info!(id, pid = child_pid, "powershell spawned");

    // Let the inner state hold the slave handle to keep it alive.
    // drop(pair.slave);

    // ── plumb the inner state ───────────────────────────────────────────────────
    let scrollback = args.max_lines.unwrap_or(1000) as usize;
    let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, scrollback)));
    let inner = Arc::new(Inner {
        id,
        title: Mutex::new(format!("Terminal {id}: {}", args.title)),
        writer: Arc::new(Mutex::new(writer)),
        _master: Mutex::new(pair.master),
        parser: Arc::clone(&parser),
        child: Mutex::new(child),
        child_pid,
        line_count: Arc::new(AtomicU32::new(0)),
        max_lines: Mutex::new(args.max_lines),
        _scrollback: scrollback,
        max_duration: Mutex::new(args.timeout_ms.map(Duration::from_millis)),
        spawn_time: Instant::now(),
        notifier: tokio::sync::broadcast::channel(16).0,
        shm: match shm::ShmBuffer::new(&format!("vterm-rs-shm-{id}"), 4096) {
            Ok(s) => { tracing::info!(id, "SHM buffer created"); Some(s) }
            Err(e) => { tracing::warn!(id, error = %e, "SHM buffer creation failed"); None }
        },
        last_content: Mutex::new(String::new()),
        full_history: Mutex::new(String::new()),
    });

    // ── start the pump (blocking thread) and viewer pipe ────────────────────────
    #[cfg(windows)]
    pump::start(Arc::clone(&inner), reader, visible);
    #[cfg(not(windows))]
    let _ = (reader, visible);

    if visible {
        spawn_viewer_window(id, &inner.title.lock());
    }

    let spawn_ms = start.elapsed().as_millis() as u64;

    // ── wait for prompt, transition to Ready, optionally send initial command ───
    let ready_start = Instant::now();
    let spawning = Terminal::<state::Spawning>::new(Arc::clone(&inner));
    let ready = prompt::wait(spawning, prompt_regex, Duration::from_secs(10)).await?;
    let ready_ms = ready_start.elapsed().as_millis() as u64;

    if let Some(initial) = args.command {
        ready.write(initial.as_bytes())?;
        ready.write(b"\r\n")?;
        
        // If we are in 'wait' mode, we need the shell itself to exit
        // so the orchestrator can detect completion.
        if args.wait.unwrap_or(false) {
            ready.write(b"exit\r\n")?;
        }
    }
    Ok(SpawnResult { terminal: ready, spawn_ms, ready_ms })
}

#[cfg(windows)]
pub(crate) fn spawn_viewer_window(id: u32, title: &str) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => { tracing::warn!(error = %e, "current_exe"); return; }
    };
    let _ = std::process::Command::new("cmd")
        .arg("/c").arg("start").arg(title)
        .arg("powershell").arg("-NoExit").arg("-Command")
        .arg(format!("& '{}' --client {}", exe.display(), id))
        .spawn();
}

#[cfg(not(windows))]
pub(crate) fn spawn_viewer_window(_id: u32, _title: &str) { /* no-op for now */ }
