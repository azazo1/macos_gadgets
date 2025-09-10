use std::ffi::OsString;
use std::fs::{self, FileType};
use std::io;

pub mod copy;
pub mod result;

pub use copy::cpcb_file;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MDirEntry {
    file_name: OsString,
    file_type: FileType,
}

impl TryFrom<&fs::DirEntry> for MDirEntry {
    type Error = io::Error;

    fn try_from(v: &fs::DirEntry) -> io::Result<Self> {
        Ok(MDirEntry {
            file_name: v.file_name(),
            file_type: v.file_type()?,
        })
    }
}

impl TryFrom<fs::DirEntry> for MDirEntry {
    type Error = io::Error;

    fn try_from(v: fs::DirEntry) -> io::Result<Self> {
        Ok(MDirEntry {
            file_name: v.file_name(),
            file_type: v.file_type()?,
        })
    }
}
