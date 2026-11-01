use crate::options::CacheMode;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

/// 下载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    /// 等待下载(包括缓存时没有请求在等待而停止的)
    Waiting,

    /// 下载中
    Downloading,

    /// 暂停中(只有永久保存的任务会暂停)
    Paused,

    /// 下载失败
    Error,

    /// 下载完成
    Finish,
}

impl DownloadState {
    pub(crate) fn from_i64(value: i64) -> Self {
        match value {
            crate::index::STATE_DOWNLOADING => DownloadState::Downloading,
            crate::index::STATE_PAUSED => DownloadState::Paused,
            crate::index::STATE_ERROR => DownloadState::Error,
            crate::index::STATE_FINISH => DownloadState::Finish,
            _ => DownloadState::Waiting,
        }
    }
}

/// 上传状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadState {
    /// 等待上传
    Waiting,

    /// 上传中(包括计算MD5)
    Uploading,

    /// 暂停中
    Paused,

    /// 上传失败
    Error,

    /// 上传完成
    Finish,

    /// 自己的文件里已经有相同的文件,跳过
    Skipped,
}

impl UploadState {
    pub(crate) fn from_i64(value: i64) -> Self {
        match value {
            crate::index::STATE_DOWNLOADING => UploadState::Uploading,
            crate::index::STATE_PAUSED => UploadState::Paused,
            crate::index::STATE_ERROR => UploadState::Error,
            crate::index::STATE_FINISH => UploadState::Finish,
            crate::index::STATE_SKIPPED => UploadState::Skipped,
            _ => UploadState::Waiting,
        }
    }

    pub(crate) fn to_i64(self) -> i64 {
        match self {
            UploadState::Waiting => crate::index::STATE_WAITING,
            UploadState::Uploading => crate::index::STATE_DOWNLOADING,
            UploadState::Paused => crate::index::STATE_PAUSED,
            UploadState::Error => crate::index::STATE_ERROR,
            UploadState::Finish => crate::index::STATE_FINISH,
            UploadState::Skipped => crate::index::STATE_SKIPPED,
        }
    }
}

/// SDK事件
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// 下载状态改变
    StateChanged {
        id: String,
        mode: CacheMode,
        state: DownloadState,

        /// 下载失败时的错误信息
        error: Option<String>,
    },

    /// 下载进度
    Progress {
        id: String,
        mode: CacheMode,

        /// 文件大小,未知时为None
        size: Option<u64>,

        /// 已经下载的大小
        downloaded: u64,

        /// 下载速度(字节/秒)
        speed: u64,
    },

    /// 缓存被删除(包括自动清理)
    Removed { ids: Vec<String> },

    /// 上传状态改变
    UploadStateChanged {
        id: i64,
        state: UploadState,

        /// 上传失败时的错误信息
        error: Option<String>,
    },

    /// 上传进度
    UploadProgress {
        id: i64,

        /// 是否正在计算MD5(计算完成之后才开始上传)
        hashing: bool,

        /// 文件大小
        size: u64,

        /// 已经处理的大小(计算MD5时为已经读取的大小)
        done: u64,

        /// 速度(字节/秒)
        speed: u64,
    },

    /// 上传记录被删除
    UploadRemoved { ids: Vec<i64> },
}

/// 事件回调,在SDK的事件线程里调用,不要在回调里做耗时的操作
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: Event);
}

/// 事件分发,所有事件在同一个线程里按顺序回调,不会阻塞下载
pub(crate) struct Events {
    listener: Arc<Mutex<Option<Arc<dyn EventListener>>>>,
    tx: Mutex<Option<Sender<Event>>>,
}

impl Events {
    pub fn new() -> Self {
        Self { listener: Arc::new(Mutex::new(None)), tx: Mutex::new(None) }
    }

    pub fn set_listener(&self, listener: Option<Arc<dyn EventListener>>) {
        *self.listener.lock().unwrap() = listener;
    }

    pub fn emit(&self, event: Event) {
        if self.listener.lock().unwrap().is_none() {
            return;
        }
        let mut tx = self.tx.lock().unwrap();
        let tx = tx.get_or_insert_with(|| {
            let (tx, rx) = channel::<Event>();
            let listener = self.listener.clone();
            std::thread::Builder::new()
                .name("dairo-file-sdk-event".to_string())
                .spawn(move || {
                    while let Ok(event) = rx.recv() {
                        let listener = listener.lock().unwrap().clone();
                        if let Some(listener) = listener {
                            listener.on_event(event);
                        }
                    }
                })
                .unwrap();
            tx
        });
        let _ = tx.send(event);
    }

    /// 发送状态改变事件
    pub fn state(&self, id: &str, mode: CacheMode, state: i64, error: Option<&str>) {
        self.emit(Event::StateChanged {
            id: id.to_string(),
            mode,
            state: DownloadState::from_i64(state),
            error: error.map(str::to_string),
        });
    }

    /// 发送上传状态改变事件
    pub fn upload_state(&self, id: i64, state: i64, error: Option<&str>) {
        self.emit(Event::UploadStateChanged { id, state: UploadState::from_i64(state), error: error.map(str::to_string) });
    }

    /// 停止事件线程
    pub fn close(&self) {
        self.tx.lock().unwrap().take();
    }
}
