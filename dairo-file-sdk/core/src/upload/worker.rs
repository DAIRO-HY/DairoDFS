use super::store::{UploadEntry, SOURCE_CUSTOM};
use super::{Account, Running, UploadSourceProvider};
use crate::error::{SdkError, SdkResult};
use crate::event::Event;
use crate::index::{STATE_FINISH, STATE_SKIPPED};
use crate::Inner;
use bytes::Bytes;
use reqwest::StatusCode;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

/// 每次读取的数据量
const READ_SIZE: u64 = 256 * 1024;

/// 计算MD5时每次读取的数据量
const HASH_READ_SIZE: u64 = 1024 * 1024;

/// 发送进度事件的间隔
const PROGRESS_INTERVAL: Duration = Duration::from_millis(500);

/// 保存上传进度的间隔
const SAVE_INTERVAL: Duration = Duration::from_secs(2);

/// 服务器繁忙时重试的间隔
const BUSY_RETRY_INTERVAL: Duration = Duration::from_secs(3);

/// 服务器繁忙时最多等待的时间
const BUSY_MAX_WAIT: Duration = Duration::from_secs(10 * 60);

/// 上传用的客户端,上传完成之后服务器要计算整个文件的MD5,等待响应的时间可能比较长
static UPLOAD_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(300))
        .build()
        .unwrap()
});

/// 上传的数据来源
trait Source: Send + Sync {
    fn size(&self) -> SdkResult<u64>;
    fn read(&self, offset: u64, len: u64) -> SdkResult<Vec<u8>>;
}

/// 本地文件
struct FileSource(Mutex<std::fs::File>);

impl Source for FileSource {
    fn size(&self) -> SdkResult<u64> {
        Ok(self.0.lock().unwrap().metadata()?.len())
    }

    fn read(&self, offset: u64, len: u64) -> SdkResult<Vec<u8>> {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = Vec::with_capacity(len as usize);
        file.by_ref().take(len).read_to_end(&mut buf)?;
        Ok(buf)
    }
}

/// 由平台提供的数据
struct CustomSource {
    provider: Arc<dyn UploadSourceProvider>,
    source: String,
}

impl Source for CustomSource {
    fn size(&self) -> SdkResult<u64> {
        self.provider.size(self.source.clone()).map_err(SdkError::Source)
    }

    fn read(&self, offset: u64, len: u64) -> SdkResult<Vec<u8>> {
        self.provider.read(self.source.clone(), offset, len).map_err(SdkError::Source)
    }
}

impl Drop for CustomSource {
    fn drop(&mut self) {
        self.provider.close(self.source.clone());
    }
}

/// 请求失败的处理方式
enum Fail {
    /// 不需要重试
    Fatal(SdkError),

    /// 网络错误等,可以重试
    Retry(SdkError),

    /// 同一个文件正在上传(可能是其他客户端,也可能是上次中断的连接服务器还没有发现),等待一段时间之后重试
    Busy(SdkError),
}

impl From<SdkError> for Fail {
    fn from(e: SdkError) -> Self {
        Fail::Fatal(e)
    }
}

/// 上传文件,返回完成之后的状态(完成或者跳过)
pub(super) async fn upload(inner: &Arc<Inner>, entry: UploadEntry, task: &Arc<Running>) -> SdkResult<i64> {
    let source: Arc<dyn Source> = if entry.source_type == SOURCE_CUSTOM {
        let provider = inner.uploads.provider().ok_or_else(|| SdkError::Source("没有设置UploadSourceProvider".to_string()))?;
        Arc::new(CustomSource { provider, source: entry.source.clone() })
    } else {
        let path = entry.source.clone();
        let file = tokio::task::spawn_blocking(move || std::fs::File::open(path))
            .await
            .map_err(|e| SdkError::Io(e.to_string()))?
            .map_err(|e| SdkError::Source(e.to_string()))?;
        Arc::new(FileSource(Mutex::new(file)))
    };
    let size = blocking({
        let source = source.clone();
        move || source.size()
    })
    .await?;

    //文件大小改变时需要重新计算MD5
    let md5 = match &entry.md5 {
        Some(md5) if entry.size.is_none_or(|it| it == size) => md5.clone(),
        _ => hash(inner, &entry, &source, size, task).await?,
    };
    inner.index.set_upload_md5(entry.id, size, &md5)?;

    let mut uploader = Uploader { inner, entry: &entry, task, source, size, md5, fails: 0 };
    let mut busy_since: Option<Instant> = None;
    loop {
        let wait = match uploader.step().await {
            Ok(Some(state)) => return Ok(state),
            Ok(None) => {
                uploader.fails = 0;
                busy_since = None;
                continue;
            }
            Err(Fail::Fatal(e)) => return Err(e),
            Err(Fail::Retry(e)) => {
                uploader.fails += 1;
                if uploader.fails > inner.config().retry_times {
                    return Err(e);
                }
                Duration::from_secs(uploader.fails.min(5) as u64)
            }
            Err(Fail::Busy(e)) => {
                if busy_since.get_or_insert_with(Instant::now).elapsed() > BUSY_MAX_WAIT {
                    return Err(e);
                }
                BUSY_RETRY_INTERVAL
            }
        };
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = task.stopped() => return Err(SdkError::Cancelled),
        }
    }
}

struct Uploader<'a> {
    inner: &'a Arc<Inner>,
    entry: &'a UploadEntry,
    task: &'a Arc<Running>,
    source: Arc<dyn Source>,
    size: u64,
    md5: String,

    /// 连续失败的次数
    fails: u32,
}

impl Uploader<'_> {
    /// 执行一轮上传
    /// - return Some(状态):上传完成;None:上传了一部分数据,继续下一轮
    async fn step(&mut self) -> Result<Option<i64>, Fail> {
        if self.entry.skip_if_exists && self.call(reqwest::Method::GET, "/api/file_upload/exists", &[("md5", &self.md5)], None).await? == "true" {
            return Ok(Some(STATE_SKIPPED));
        }

        //秒传,文件还没有上传时返回404
        let content_type = self.entry.content_type.clone().unwrap_or_default();
        let params = [("md5", self.md5.as_str()), ("path", self.entry.dfs_path.as_str()), ("contentType", content_type.as_str())];
        match self.call(reqwest::Method::POST, "/api/file_upload/by_md5", &params, None).await {
            Ok(_) => return Ok(Some(STATE_FINISH)),
            Err(Fail::Fatal(SdkError::NotFound)) => {}
            //路径不正确等业务错误,不需要重试
            Err(Fail::Retry(SdkError::Server(msg))) => return Err(Fail::Fatal(SdkError::Server(msg))),
            Err(e) => return Err(e),
        }

        //断点续传,服务器上的临时文件已经完整时上传空数据,服务器会校验并保存
        let uploaded = self.call(reqwest::Method::GET, "/api/file_upload/uploaded_size", &[("md5", &self.md5)], None).await?;
        let offset: u64 = uploaded.trim().parse().map_err(|_| Fail::Retry(SdkError::Server(uploaded.clone())))?;
        self.stream(offset.min(self.size)).await?;
        Ok(None)
    }

    /// 从offset开始上传剩下的数据
    async fn stream(&mut self, offset: u64) -> Result<(), Fail> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(4);
        let source = self.source.clone();
        let size = self.size;
        let task = self.task.clone();

        //在后台线程里读取数据
        self.inner.rt.spawn_blocking(move || {
            let mut pos = offset;
            while pos < size && !task.is_stopped() {
                let data = source.read(pos, READ_SIZE.min(size - pos)).map_err(std::io::Error::other);
                let len = data.as_ref().map(|it| it.len() as u64).unwrap_or(0);
                let failed = data.is_err() || len == 0;
                let data = if len == 0 && data.is_ok() { Err(std::io::Error::other("读取的数据比文件大小少")) } else { data };
                if tx.blocking_send(data.map(Bytes::from)).is_err() || failed {
                    return;
                }
                pos += len;
            }
        });

        //统计已经交给网络发送的数据量
        let sent = Arc::new(AtomicU64::new(offset));
        let progress = Progress::new(self.inner, self.entry.id, size, offset);
        let stream = {
            let sent = sent.clone();
            let progress = Mutex::new(progress);
            let task = self.task.clone();
            futures_util::stream::poll_fn(move |cx| {
                //请求体由连接的后台任务发送,不再等待响应时也会继续发送,所以停止时要让请求体返回错误,连接才会断开
                if task.is_stopped() {
                    return std::task::Poll::Ready(Some(Err(std::io::Error::other("上传已停止"))));
                }
                let item = rx.poll_recv(cx);
                if let std::task::Poll::Ready(Some(Ok(data))) = &item {
                    let done = sent.fetch_add(data.len() as u64, Ordering::Relaxed) + data.len() as u64;
                    progress.lock().unwrap().update(done, false);
                }
                item
            })
        };
        let url = format!("/api/file_upload/by_stream/{}", self.md5);
        let body = reqwest::Body::wrap_stream(stream);
        let result = self.call(reqwest::Method::POST, &url, &[], Some((body, size - offset))).await;
        let _ = self.inner.index.set_upload_progress(self.entry.id, sent.load(Ordering::Relaxed));
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                //上传了一部分数据之后网络中断,重新计算失败次数
                if sent.load(Ordering::Relaxed) > offset {
                    self.fails = 0;
                }
                Err(e)
            }
        }
    }

    /// 请求DFS接口
    /// - body 请求体和长度
    async fn call(&self, method: reqwest::Method, path: &str, params: &[(&str, &str)], body: Option<(reqwest::Body, u64)>) -> Result<String, Fail> {
        let account = self.account()?;
        let mut req = UPLOAD_CLIENT
            .request(method, format!("{}{}", account.server, path))
            .bearer_auth(&account.token)
            .query(params);
        if let Some((body, len)) = body {
            req = req
                .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                .header(reqwest::header::CONTENT_LENGTH, len)
                .body(body);
        }
        let send = req.send();
        tokio::pin!(send);
        let resp = tokio::select! {
            it = &mut send => it,
            _ = self.task.stopped() => {
                //停止之后请求体会返回错误,继续等一会儿让连接正常断开,否则服务器会一直等待剩下的数据
                let _ = tokio::time::timeout(Duration::from_secs(2), &mut send).await;
                return Err(Fail::Fatal(SdkError::Cancelled));
            }
        };
        let resp = resp.map_err(|e| Fail::Retry(SdkError::Network(e.to_string())))?;
        let status = resp.status();
        let text = tokio::select! {
            it = resp.text() => it.map_err(|e| Fail::Retry(SdkError::Network(e.to_string())))?,
            _ = self.task.stopped() => return Err(Fail::Fatal(SdkError::Cancelled)),
        };
        match status {
            StatusCode::OK => Ok(text),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(Fail::Fatal(SdkError::UrlExpired(status.as_u16()))),
            StatusCode::NOT_FOUND => Err(Fail::Fatal(SdkError::NotFound)),
            StatusCode::CONFLICT => Err(Fail::Busy(SdkError::Server(text))),
            //上传中断或者校验失败(临时文件已经被删除)等,重试时会重新查询已经上传的大小
            StatusCode::INTERNAL_SERVER_ERROR => Err(Fail::Retry(SdkError::Server(text))),
            _ if status.is_server_error() => Err(Fail::Retry(SdkError::Http(status.as_u16()))),
            _ => Err(Fail::Fatal(SdkError::Http(status.as_u16()))),
        }
    }

    /// 当前账号,账号已经切换时停止上传
    fn account(&self) -> Result<Account, Fail> {
        match self.inner.uploads.account() {
            Some(it) if it.name == self.entry.account => Ok(it),
            _ => Err(Fail::Fatal(SdkError::Cancelled)),
        }
    }
}

/// 计算MD5
async fn hash(inner: &Arc<Inner>, entry: &UploadEntry, source: &Arc<dyn Source>, size: u64, task: &Arc<Running>) -> SdkResult<String> {
    let source = source.clone();
    let task = task.clone();
    let mut progress = Progress::new(inner, entry.id, size, 0);
    blocking(move || {
        let mut context = md5::Context::new();
        let mut pos = 0;
        while pos < size {
            if task.is_stopped() {
                return Err(SdkError::Cancelled);
            }
            let data = source.read(pos, HASH_READ_SIZE.min(size - pos))?;
            if data.is_empty() {
                return Err(SdkError::Source("读取的数据比文件大小少".to_string()));
            }
            context.consume(&data);
            pos += data.len() as u64;
            progress.update(pos, true);
        }
        Ok(format!("{:x}", context.finalize()))
    })
    .await
}

/// 上传进度通知
struct Progress {
    inner: Arc<Inner>,
    id: i64,
    size: u64,
    last_time: Instant,
    last_done: u64,
    last_save: Instant,
}

impl Progress {
    fn new(inner: &Arc<Inner>, id: i64, size: u64, done: u64) -> Self {
        Self { inner: inner.clone(), id, size, last_time: Instant::now(), last_done: done, last_save: Instant::now() }
    }

    fn update(&mut self, done: u64, hashing: bool) {
        let elapsed = self.last_time.elapsed();
        if elapsed < PROGRESS_INTERVAL && done < self.size {
            return;
        }
        let speed = (done.saturating_sub(self.last_done) as f64 / elapsed.as_secs_f64().max(0.001)) as u64;
        self.last_time = Instant::now();
        self.last_done = done;
        self.inner.events.emit(Event::UploadProgress { id: self.id, hashing, size: self.size, done, speed });
        if !hashing && self.last_save.elapsed() >= SAVE_INTERVAL {
            self.last_save = Instant::now();
            let _ = self.inner.index.set_upload_progress(self.id, done);
        }
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> SdkResult<T> + Send + 'static) -> SdkResult<T> {
    tokio::task::spawn_blocking(f).await.map_err(|e| SdkError::Io(e.to_string()))?
}
