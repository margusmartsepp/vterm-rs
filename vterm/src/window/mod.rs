//! Window control. The only OS-coupled module.
//!
//! Resolves a child process PID to its top-level visible HWND via `EnumWindows` +
//! `GetWindowThreadProcessId` — robust against shell title changes (a frequent v0.5
//! bug source). Non-Windows targets compile to a stub that returns a typed error.

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod stub;

#[cfg(windows)]
pub use self::windows::{control, set_title, show};
#[cfg(not(windows))]
pub use self::stub::{control, set_title, show};
