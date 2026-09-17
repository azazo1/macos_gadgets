use crate::result::Result;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, FileType};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::debug;

/// 目录项标识: 文件名加文件类型, 用来判断文件是不是本轮新出现的.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct MDirEntry {
    file_name: OsString,
    file_type: FileType,
}

impl TryFrom<&fs::DirEntry> for MDirEntry {
    type Error = std::io::Error;

    fn try_from(entry: &fs::DirEntry) -> std::io::Result<Self> {
        Ok(MDirEntry {
            file_name: entry.file_name(),
            file_type: entry.file_type()?,
        })
    }
}

/// 下载目录监视器.
///
/// 单次读取失败 (条目被删除, 权限不足等) 只会跳过该条目, 不会中断扫描.
pub struct DownloadWatcher {
    dir: PathBuf,
    ignore_exts: HashSet<String>,
    known: HashSet<MDirEntry>,
    initialized: bool,
}

impl DownloadWatcher {
    /// `ignore_exts` 中列出的扩展名会被忽略, 例如下载工具的中间文件.
    pub fn new(
        dir: impl Into<PathBuf>,
        ignore_exts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            dir: dir.into(),
            ignore_exts: ignore_exts.into_iter().map(Into::into).collect(),
            known: HashSet::new(),
            initialized: false,
        }
    }

    /// 扫描一次下载目录, 返回相对上次扫描新增的文件, 按访问时间从旧到新排列.
    ///
    /// 首次扫描只建立基线, 返回空列表. 扫描失败时基线保持不变, 下次调用继续比较.
    pub fn scan_new_files(&mut self) -> Result<Vec<PathBuf>> {
        let mut known = HashSet::new();
        let mut new_files: Vec<(Option<SystemTime>, PathBuf)> = Vec::new();

        for entry in fs::read_dir(&self.dir)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    debug!("skip unreadable download dir entry: {err}");
                    continue;
                }
            };
            let key = match MDirEntry::try_from(&entry) {
                Ok(key) => key,
                Err(err) => {
                    debug!("skip {}: {err}", entry.path().display());
                    continue;
                }
            };
            let is_new = self.initialized && !self.known.contains(&key);
            known.insert(key);
            if !is_new {
                continue;
            }
            let path = entry.path();
            if self.is_ignored(&path) {
                debug!("ignore {}", path.display());
                continue;
            }
            // 访问时间只用于排序, 取不到时排在最前面.
            let accessed = entry.metadata().ok().and_then(|meta| meta.accessed().ok());
            new_files.push((accessed, path));
        }

        self.known = known;
        self.initialized = true;
        new_files.sort_by_key(|(accessed, _)| *accessed);
        Ok(new_files.into_iter().map(|(_, path)| path).collect())
    }

    fn is_ignored(&self, path: &Path) -> bool {
        match path.extension() {
            Some(ext) => self.ignore_exts.contains(ext.to_string_lossy().as_ref()),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "watch_downloads_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn scan_missing_dir_reports_error() {
        let mut watcher = DownloadWatcher::new("/nonexistent/watch_downloads_dir", ["aria2"]);
        assert!(watcher.scan_new_files().is_err());
    }

    #[test]
    fn scan_reports_only_new_files() {
        let dir = temp_dir("new_files");
        fs::write(dir.join("old.txt"), b"old").expect("write old file");

        let mut watcher = DownloadWatcher::new(&dir, ["aria2"]);
        assert!(watcher.scan_new_files().expect("baseline scan").is_empty());

        fs::write(dir.join("new.txt"), b"new").expect("write new file");
        fs::write(dir.join("new.txt.aria2"), b"tmp").expect("write temp file");
        assert_eq!(
            watcher.scan_new_files().expect("second scan"),
            vec![dir.join("new.txt")]
        );
        // 已经报告过的文件不再重复报告.
        assert!(watcher.scan_new_files().expect("third scan").is_empty());

        fs::remove_dir_all(&dir).expect("clean temp dir");
    }
}
