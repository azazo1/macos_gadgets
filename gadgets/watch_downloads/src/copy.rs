use crate::result::*;
use clipboard_rs::{Clipboard, ClipboardContext};
use std::path::Path;

/// copy file to clipboard.
pub fn cpcb_file(p: impl AsRef<Path>) -> Result<()> {
    let ctx = ClipboardContext::new()
        .map_err(|_| Error::ClipboardError("failed to create clipboard context".into()))?;
    ctx.set_files(vec![url::Url::from_file_path(p).unwrap().as_str().into()])
        .map_err(|_| Error::ClipboardError("failed to copy file".into()))?;
    Ok(())
}
