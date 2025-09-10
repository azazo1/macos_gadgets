use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
