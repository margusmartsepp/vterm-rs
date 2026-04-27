//! Shortcut → byte translation.
//!
//! Turns human-readable keystroke tokens (`<C-c>`, `<Up>`, `<Esc>`, `<Enter>`) into the
//! exact byte sequences a PTY-attached shell expects. Anything that doesn't match a
//! known token is passed through verbatim, so `"hello<Enter>"` becomes
//! `b"hello\r\n"` and `"<unknown>"` becomes `b"<unknown>"`.
//!
//! The regex is compiled exactly once via [`std::sync::OnceLock`] — no `lazy_static`
//! macro, no per-call cost.

use regex::Regex;
use std::sync::OnceLock;

fn token_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<([A-Za-z0-9-]+)>").expect("static regex"))
}

/// Translate `text` into raw bytes, expanding shortcut tokens.
pub fn parse(text: &str) -> Vec<u8> {
    let re = token_re();
    let mut out = Vec::with_capacity(text.len());
    let mut cursor = 0usize;

    for cap in re.captures_iter(text) {
        let m = cap.get(0).expect("group 0 exists");
        out.extend_from_slice(&text.as_bytes()[cursor..m.start()]);

        let token = cap.get(1).expect("group 1 exists").as_str();
        match expand(token) {
            Some(bytes) => out.extend_from_slice(bytes),
            None => out.extend_from_slice(m.as_str().as_bytes()), // unknown — pass through
        }

        cursor = m.end();
    }
    out.extend_from_slice(&text.as_bytes()[cursor..]);
    out
}

/// Expansion table. `None` means "unknown token; pass through verbatim".
fn expand(token: &str) -> Option<&'static [u8]> {
    // Tokens are case-insensitive — match against a lowercased view via byte trick.
    // For three-byte tokens we can avoid the allocation entirely.
    Some(match token.to_ascii_lowercase().as_str() {
        "c-c"   => &[0x03],
        "c-d"   => &[0x04],
        "tab"   => &[0x09],
        "esc"   => &[0x1b],
        "enter" => b"\r\n",
        "up"    => b"\x1b[A",
        "down"  => b"\x1b[B",
        "right" => b"\x1b[C",
        "left"  => b"\x1b[D",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn ctrl_c() { assert_eq!(parse("<C-c>"), &[0x03]); }
    #[test] fn arrow_up() { assert_eq!(parse("<Up>"), b"\x1b[A"); }
    #[test] fn enter() { assert_eq!(parse("<Enter>"), b"\r\n"); }
    #[test] fn esc() { assert_eq!(parse("<Esc>"), b"\x1b"); }
    #[test] fn mixed() { assert_eq!(parse("ls -la<Enter>"), b"ls -la\r\n"); }
    #[test] fn unknown_passes_through() { assert_eq!(parse("<floop>"), b"<floop>"); }
    #[test] fn case_insensitive() { assert_eq!(parse("<ENTER>"), b"\r\n"); }
    #[test] fn vim_quit() { assert_eq!(parse("<Esc>:q!<Enter>"), b"\x1b:q!\r\n"); }
}
