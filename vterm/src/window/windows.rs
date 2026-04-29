//! Windows-specific window control: PID → HWND, then the requested action.

use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::{Error, Result};

/// Apply `action` to the top-level visible window owned by `pid`.
pub fn control(pid: u32, action: &str) -> Result<()> {
    let hwnd = find_hwnd_for_pid(pid)
        .ok_or_else(|| Error::Window(format!("no top-level window for PID {pid}")))?;

    // SAFETY: HWND is non-null (find_hwnd_for_pid guarantees), action constants are
    // FFI-safe, and these calls do not invalidate any Rust references.
    unsafe {
        match action {
            "minimize" => {
                ShowWindow(hwnd, SW_MINIMIZE);
            }
            "maximize" => {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
            "restore" => {
                ShowWindow(hwnd, SW_RESTORE);
            }
            "close" => {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            "pin" => {
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
            "unpin" => {
                SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
            "menu" => {
                PostMessageW(hwnd, WM_SYSCOMMAND, SC_KEYMENU as usize, 0);
            }
            "show" => {
                ShowWindow(hwnd, SW_SHOW);
            }
            "hide" => {
                ShowWindow(hwnd, SW_HIDE);
            }
            other => return Err(Error::Window(format!("unknown action `{other}`"))),
        }
    }
    Ok(())
}

/// Dynamically update the window title for `pid`.
pub fn set_title(pid: u32, title: &str) -> Result<()> {
    let hwnd = find_hwnd_for_pid(pid)
        .ok_or_else(|| Error::Window(format!("no top-level window for PID {pid}")))?;

    // SAFETY: title is converted to wide string for FFI.
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = std::ffi::OsStr::new(title).encode_wide().collect();
        wide.push(0);
        SetWindowTextW(hwnd, wide.as_ptr());
    }
    Ok(())
}

/// Make the window visible for `pid`.
pub fn show(pid: u32) -> Result<()> {
    control(pid, "show")
}

/// Walk the window list and return the first top-level *visible* window owned by `pid`.
fn find_hwnd_for_pid(pid: u32) -> Option<HWND> {
    struct Search {
        pid: u32,
        hwnd: HWND,
    }
    let mut state = Search { pid, hwnd: 0 };

    // SAFETY: `state` outlives the callback (we block on EnumWindows).
    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = unsafe { &mut *(lparam as *mut Search) };
        let mut owner: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut owner) };
        if owner == state.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            state.hwnd = hwnd;
            return FALSE; // stop enumerating
        }
        TRUE
    }

    unsafe {
        EnumWindows(Some(cb), &mut state as *mut Search as LPARAM);
    }
    (state.hwnd != 0).then_some(state.hwnd)
}
