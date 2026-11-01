//! 文件下载缓存SDK
//!
//! - 同一个文件的多个请求合并成一次下载,下载完成或失败时所有请求都能收到结果
//! - 断点续传,App重启之后也能从上次的位置继续下载
//! - 缓存的key去掉了域名部分,服务器切换域名之后缓存依然有效,通过命名空间区分不同的服务器

mod access;
mod cleaner;
mod config;
mod download;
mod error;
mod event;
mod index;
mod key;
mod manage;
mod mime;
mod options;
mod range_set;
mod server;
mod upload;

pub use config::Config;
pub use error::{SdkError, SdkResult};
pub use event::{DownloadState, Event, EventListener, UploadState};
pub use manage::{CacheInfo, CacheQuery, GroupStats};
pub use upload::{UploadInfo, UploadItem, UploadQuery, UploadSource, UploadSourceProvider};
pub use options::{CacheMode, KeyMode, Options};

use access::AccessRecorder;
use download::{Acquired, Downloads};
use index::{Entry, Index, STATE_DOWNLOADING, STATE_FINISH, STATE_WAITING};
use server::Server;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::{Handle, Runtime};
use tokio::task::AbortHandle;

/// 下载结果回调
pub trait FetchListener: Send + Sync {
    /// 下载进度
    /// - size 文件大小,未知时为None
    /// - downloaded 已经下载的大小
    fn on_progress(&self, size: Option<u64>, downloaded: u64) {
        let _ = (size, downloaded);
    }

    /// 下载完成
    /// - path 缓存文件路径
    fn on_success(&self, path: String);

    /// 下载失败
    fn on_error(&self, error: SdkError);
}

/// 一次下载请求,可以取消
pub struct FetchTask {
    abort: AbortHandle,
}

impl FetchTask {
    /// 取消请求,没有其他请求在等待同一个文件时会停止下载(永久保存的任务不会停止),取消之后不会再回调
    pub fn cancel(&self) {
        self.abort.abort();
    }
}

/// 存入缓存的数据
pub enum PutSource {
    /// 复制一个本地文件
    Path(String),

    /// 直接写入数据
    Bytes(Vec<u8>),
}

pub(crate) struct Inner {
    pub config: Mutex<Config>,
    pub root: PathBuf,
    pub index: Index,
    pub client: reqwest::Client,
    pub downloads: Downloads,
    pub access: AccessRecorder,
    pub server: Server,
    pub events: event::Events,
    pub cleaner: cleaner::Cleaner,
    pub uploads: upload::Uploads,
    pub rt: Handle,
}

impl Inner {
    /// 当前配置(部分配置可以在运行时修改)
    pub fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }
}

/// 文件SDK,一个数据目录只能创建一个实例
pub struct FileSdk {
    pub(crate) inner: Arc<Inner>,
    runtime: Option<Runtime>,
}

impl FileSdk {
    pub fn new(config: Config) -> SdkResult<Self> {
        let root = PathBuf::from(&config.root_dir);
        std::fs::create_dir_all(root.join("files"))?;
        let index = Index::open(&root.join("index.sqlite"))?;
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .read_timeout(Duration::from_secs(config.read_timeout))
            .build()
            .map_err(|e| SdkError::Network(e.to_string()))?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("dairo-file-sdk")
            .enable_all()
            .build()?;
        let inner = Arc::new(Inner {
            config: Mutex::new(config),
            root,
            index,
            client,
            downloads: Downloads::default(),
            access: AccessRecorder::default(),
            server: Server::default(),
            events: event::Events::new(),
            cleaner: cleaner::Cleaner::default(),
            uploads: upload::Uploads::default(),
            rt: runtime.handle().clone(),
        });
        init(&inner)?;
        Ok(Self { inner, runtime: Some(runtime) })
    }

    /// 下载文件,完成之后通过回调返回缓存文件路径(已经缓存时立即回调)
    pub fn fetch(&self, url: &str, options: Options, listener: Arc<dyn FetchListener>) -> FetchTask {
        let inner = self.inner.clone();
        let url = url.to_string();
        let handle = self.inner.rt.spawn(async move {
            match fetch(&inner, &url, &options, Some(listener.as_ref())).await {
                Ok(path) => listener.on_success(path),
                Err(SdkError::Cancelled) => {}
                Err(e) => listener.on_error(e),
            }
        });
        FetchTask { abort: handle.abort_handle() }
    }

    /// 下载文件,完成之后返回缓存文件路径
    pub async fn fetch_async(&self, url: &str, options: Options) -> SdkResult<String> {
        let inner = self.inner.clone();
        let url = url.to_string();

        //在SDK自己的运行时里执行,调用方可能不是tokio
        self.inner
            .rt
            .spawn(async move { fetch(&inner, &url, &options, None).await })
            .await
            .map_err(|_| SdkError::Cancelled)?
    }

    /// 下载文件,完成之后返回缓存文件路径(阻塞当前线程,不要在主线程调用)
    pub fn fetch_blocking(&self, url: &str, options: Options) -> SdkResult<String> {
        self.inner.rt.block_on(self.fetch_async(url, options))
    }

    /// 获取已经缓存的文件路径,没有缓存时返回None
    pub fn cached_path(&self, url: &str, options: &Options) -> SdkResult<Option<String>> {
        let (_, key) = key::resolve(url, options)?;
        let Some(entry) = self.inner.index.get(&key.id)? else {
            return Ok(None);
        };
        if entry.state != STATE_FINISH {
            return Ok(None);
        }
        let path = self.inner.root.join(entry.file());
        if !path.exists() {
            return Ok(None);
        }
        self.inner.access.record(&self.inner.index, &entry.id);
        Ok(Some(path.to_string_lossy().to_string()))
    }

    /// 把本地的数据存入缓存,之后可以像下载过的文件一样访问
    pub fn put_file(&self, url: &str, options: &Options, source: PutSource) -> SdkResult<String> {
        let (parsed, key) = key::resolve(url, options)?;
        let now = now_millis();
        let entry = Entry {
            ext: key::ext(&parsed, options.name.as_deref()),
            id: key.id,
            namespace: key.namespace,
            key: key.key,
            url: url.to_string(),
            group: options.group.clone(),
            mode: options.cache,
            name: options.name.clone(),
            thumb: options.thumb.clone(),
            size: None,
            ranges: Default::default(),
            etag: None,
            last_modified: None,
            content_type: None,
            md5: options.md5.clone(),
            headers: options.headers.clone(),
            state: STATE_FINISH,
            error: None,
            create_date: now,
            finish_date: Some(now),
            access_date: now,
        };
        let mut entry = match self.inner.index.get(&entry.id)? {
            Some(old) => Entry { create_date: old.create_date, mode: old.mode.max(entry.mode), ..entry },
            None => entry,
        };
        let path = self.inner.root.join(entry.file());
        std::fs::create_dir_all(path.parent().unwrap())?;

        //先写临时文件再改名,避免写到一半时被当成完整的文件
        let temp = download::part_path(&path);
        let size = match source {
            PutSource::Path(from) => std::fs::copy(from, &temp)?,
            PutSource::Bytes(data) => {
                std::fs::write(&temp, &data)?;
                data.len() as u64
            }
        };
        std::fs::rename(&temp, &path)?;
        entry.size = Some(size);
        entry.ranges.insert(0, size);
        self.inner.index.save(&entry)?;
        self.inner.events.state(&entry.id, entry.mode, STATE_FINISH, None);
        Ok(path.to_string_lossy().to_string())
    }

    /// 登记一个下载地址,返回本机HTTP访问地址,交给播放器、图片组件等使用
    /// 地址里不会带有下载地址和请求头,登记的信息只在本次运行期间有效
    pub fn open(&self, url: &str, options: &Options) -> SdkResult<String> {
        self.inner.server.open(&self.inner, url, options)
    }

    /// 生成无状态的本机HTTP访问地址(参数和下载地址编码在地址里),适合列表里大量的缩略图
    pub fn local_url(&self, url: &str, options: &Options) -> SdkResult<String> {
        self.inner.server.local_url(&self.inner, url, options)
    }

    /// 本机HTTP访问地址的前缀,平台可以用它自己拼接无状态地址
    pub fn local_base(&self) -> SdkResult<String> {
        self.inner.server.base(&self.inner)
    }

    /// 检查本机HTTP服务,不能访问时重新启动,返回监听的端口
    /// iOS从后台回到前台时调用,重启之后优先使用原来的端口,之前的地址依然有效
    pub fn check_server(&self) -> SdkResult<u16> {
        self.inner.server.check(&self.inner)
    }

    /// 停止本机HTTP服务,下次访问时会自动启动
    pub fn stop_server(&self) {
        self.inner.server.stop();
    }

    /// 把内存中记录的访问时间写入数据库,App进入后台时调用
    pub fn flush(&self) {
        self.inner.access.flush(&self.inner.index);
    }
}

impl Drop for FileSdk {
    fn drop(&mut self) {
        self.flush();
        self.inner.server.stop();
        self.inner.events.close();
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

/// 启动时的初始化
fn init(inner: &Arc<Inner>) -> SdkResult<()> {
    //删除上次运行时不缓存的文件
    for it in inner.index.select_by_mode(CacheMode::None)? {
        cleaner::remove_files(inner, &it);
        inner.index.delete(&it.id)?;
    }

    //上次运行时没有下载完成的任务
    for it in inner.index.select_by_mode(CacheMode::Cache)? {
        if it.state == STATE_DOWNLOADING {
            inner.index.set_state(&it.id, STATE_WAITING, None)?;
        }
    }
    upload::init(inner)?;
    for it in inner.index.select_by_mode(CacheMode::Save)? {
        if it.state == STATE_WAITING || it.state == STATE_DOWNLOADING {
            inner.downloads.resume_save(inner, &it);
        }
    }

    //定时把访问时间写入数据库
    let weak = Arc::downgrade(inner);
    inner.rt.spawn(async move {
        loop {
            tokio::time::sleep(access::MAX_PENDING_TIME).await;
            let Some(inner) = weak.upgrade() else {
                return;
            };
            inner.access.flush(&inner.index);
        }
    });

    //启动时和之后定时清理缓存
    let weak = Arc::downgrade(inner);
    inner.rt.spawn(async move {
        loop {
            let Some(inner) = weak.upgrade() else {
                return;
            };
            cleaner::spawn_clean(&inner);
            drop(inner);
            tokio::time::sleep(cleaner::CLEAN_INTERVAL).await;
        }
    });
    Ok(())
}

/// 回调下载进度的间隔
const LISTENER_PROGRESS_INTERVAL: Duration = Duration::from_millis(200);

/// 请求文件并等待下载完成
async fn fetch(inner: &Arc<Inner>, url: &str, options: &Options, listener: Option<&dyn FetchListener>) -> SdkResult<String> {
    let waiter = match inner.downloads.acquire(inner, url, options)? {
        Acquired::Ready(_, path) => return Ok(path.to_string_lossy().to_string()),
        Acquired::Waiting(it) => it,
    };
    let mut rx = waiter.task.subscribe();
    let mut last_notify: Option<Instant> = None;
    loop {
        let (started, size, downloaded, result) = {
            let data = rx.borrow_and_update();
            (data.started, data.size, data.ranges.len(), data.result.clone())
        };
        if let Some(result) = result {
            return result;
        }
        if started
            && let Some(listener) = listener
            && last_notify.is_none_or(|it| it.elapsed() >= LISTENER_PROGRESS_INTERVAL)
        {
            last_notify = Some(Instant::now());
            listener.on_progress(size, downloaded);
        }
        if rx.changed().await.is_err() {
            return Err(SdkError::Cancelled);
        }
    }
}

/// 当前时间戳(毫秒)
pub(crate) fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|it| it.as_millis() as i64)
        .unwrap_or(0)
}
