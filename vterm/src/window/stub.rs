//! Non-Windows stub. Returns a typed error so callers can surface it properly.

use crate::{Error, Result};

pub fn control(_pid: u32, _action: &str) -> Result<()> {
    Err(Error::Window("window control not yet supported on this platform".into()))
}
