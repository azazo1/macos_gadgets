use std::io;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("clipboard error: {0}")]
    Clipboard(String),
    #[error("path is not valid utf-8: {0}")]
    InvalidPath(PathBuf),
    #[error("download directory (~/Downloads) is unavailable")]
    DownloadDirUnavailable,
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
