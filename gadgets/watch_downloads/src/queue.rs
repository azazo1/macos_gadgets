use crate::copy::cpcb_file;
use std::collections::VecDeque;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// 同一个文件最多经历多少轮复制尝试, 超过后放弃并记录错误.
const MAX_ROUNDS: u32 = 3;

struct Pending {
    path: PathBuf,
    rounds: u32,
}

/// 待复制队列.
///
/// 写入剪贴板失败的文件不会被丢弃, 而是留在队列里等下一轮继续重试,
/// 直到成功或者超出轮数上限.
#[derive(Default)]
pub struct CopyQueue {
    pending: VecDeque<Pending>,
}

impl CopyQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 加入待复制的文件, 队列里已有的相同路径会被忽略.
    pub fn enqueue(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        if self.pending.iter().any(|item| item.path == path) {
            return;
        }
        self.pending.push_back(Pending { path, rounds: 0 });
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// 尝试复制队列中的全部文件, 每调用一次算一轮, 失败的文件留到下一轮.
    pub fn flush_round(&mut self) {
        let mut retry = VecDeque::new();
        for mut item in self.pending.drain(..) {
            match cpcb_file(&item.path) {
                Ok(()) => info!("copied: {}", item.path.display()),
                Err(err) => {
                    item.rounds += 1;
                    if item.rounds >= MAX_ROUNDS {
                        error!(
                            "give up {} after {MAX_ROUNDS} rounds: {err}",
                            item.path.display()
                        );
                    } else {
                        warn!(
                            "copy {} failed, retry in next round ({}/{MAX_ROUNDS}): {err}",
                            item.path.display(),
                            item.rounds
                        );
                        retry.push_back(item);
                    }
                }
            }
        }
        self.pending = retry;
    }
}
