use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use watch_downloads::result::{Error, Result};
use watch_downloads::{CopyQueue, DownloadWatcher};

/// 轮询下载目录的间隔.
const POLL_INTERVAL: Duration = Duration::from_secs(1);
/// 需要忽略的下载中间文件扩展名.
const IGNORE_EXTS: [&str; 1] = ["aria2"];

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(err) = run() {
        error!("watch_downloads exit: {err}");
        std::process::exit(1);
    }
}

/// 主循环: 只处理真正无法继续的错误, 其余失败都重试, 不退出进程.
fn run() -> Result<()> {
    let download_dir = dirs::download_dir().ok_or(Error::DownloadDirUnavailable)?;
    info!(
        "watch_downloads launched, watching {}.",
        download_dir.display()
    );

    let mut watcher = DownloadWatcher::new(download_dir, IGNORE_EXTS);
    let mut queue = CopyQueue::new();
    let mut scan_failed = false;

    loop {
        match watcher.scan_new_files() {
            Ok(new_files) => {
                if scan_failed {
                    info!("download directory readable again.");
                    scan_failed = false;
                }
                for path in new_files {
                    debug!("new file: {}", path.display());
                    queue.enqueue(path);
                }
            }
            Err(err) => {
                // 目录暂时不可用 (被移动, 未挂载等) 时保持运行, 下一轮再试.
                if scan_failed {
                    debug!("scan download directory failed: {err}");
                } else {
                    warn!("scan download directory failed, retrying: {err}");
                    scan_failed = true;
                }
            }
        }
        queue.flush_round();
        thread::sleep(POLL_INTERVAL);
    }
}
