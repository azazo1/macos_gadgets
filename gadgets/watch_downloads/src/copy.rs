use crate::result::{Error, Result};
use clipboard_rs::{Clipboard, ClipboardContext};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::debug;

/// 单次复制到剪贴板的最大尝试次数.
const MAX_ATTEMPTS: u32 = 3;
/// 首次重试前的等待时间, 之后每次翻倍.
const RETRY_BACKOFF: Duration = Duration::from_millis(200);

/// 把文件复制到剪贴板, 写入失败时按指数退避重试, 全部失败才返回错误.
///
/// 这是后台常驻程序调用的入口, 任何失败都只以错误形式返回, 不会 panic.
pub fn cpcb_file(p: impl AsRef<Path>) -> Result<()> {
    let path = p.as_ref();
    let mut backoff = RETRY_BACKOFF;
    let mut attempt = 1;
    loop {
        match cpcb_file_once(path) {
            Ok(()) => return Ok(()),
            Err(err) => {
                if attempt >= MAX_ATTEMPTS {
                    return Err(err);
                }
                debug!(
                    "clipboard write of {} failed on attempt {attempt}/{MAX_ATTEMPTS}: \
                     {err}, retry in {backoff:?}",
                    path.display()
                );
                thread::sleep(backoff);
                backoff *= 2;
                attempt += 1;
            }
        }
    }
}

/// 把文件写入剪贴板, 只尝试一次.
fn cpcb_file_once(path: &Path) -> Result<()> {
    // 剪贴板需要的是本地文件路径本身, file:// URL 会写入错误的文件名.
    let file = path
        .to_str()
        .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?;
    let ctx = ClipboardContext::new()
        .map_err(|err| Error::Clipboard(format!("failed to create clipboard context: {err}")))?;
    ctx.set_files(vec![file.to_string()]).map_err(|err| {
        Error::Clipboard(format!(
            "failed to write {} to clipboard: {err}",
            path.display()
        ))
    })?;
    Ok(())
}
