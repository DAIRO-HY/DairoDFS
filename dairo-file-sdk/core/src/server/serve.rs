use crate::download::{Acquired, Task, TaskData, Waiter};
use crate::error::SdkError;
use crate::options::Options;
use crate::{mime, Inner};
use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full, StreamBody};
use hyper::body::Frame;
use hyper::{Response, StatusCode};
use std::io::SeekFrom;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::watch;

pub type Body = BoxBody<Bytes, std::io::Error>;

/// 每次最多读取的数据量
const READ_CHUNK_SIZE: u64 = 64 * 1024;

/// 请求头里的Range
#[derive(Debug, Clone, Copy, PartialEq)]
enum RangeSpec {
    /// 整个文件
    All,

    /// 从开始位置到结束位置(包含),没有结束位置时到文件末尾
    From(u64, Option<u64>),

    /// 最后n个字节
    Suffix(u64),
}

/// 只支持单个区间,多个区间或者格式不正确时返回整个文件
fn parse_range(value: Option<&str>) -> RangeSpec {
    let Some(value) = value.and_then(|it| it.trim().strip_prefix("bytes=")) else {
        return RangeSpec::All;
    };
    if value.contains(',') {
        return RangeSpec::All;
    }
    let Some((start, end)) = value.split_once('-') else {
        return RangeSpec::All;
    };
    let (start, end) = (start.trim(), end.trim());
    if start.is_empty() {
        return end.parse().map(RangeSpec::Suffix).unwrap_or(RangeSpec::All);
    }
    let Ok(start) = start.parse() else {
        return RangeSpec::All;
    };
    if end.is_empty() {
        return RangeSpec::From(start, None);
    }
    match end.parse() {
        Ok(end) if end >= start => RangeSpec::From(start, Some(end)),
        _ => RangeSpec::All,
    }
}

/// 计算要返回的区间[开始, 结束),None表示返回整个文件,Err表示区间不合法(416)
fn resolve_range(spec: RangeSpec, size: u64) -> Result<Option<(u64, u64)>, ()> {
    match spec {
        RangeSpec::All => Ok(None),
        RangeSpec::From(start, _) if start >= size => Err(()),
        RangeSpec::From(start, end) => Ok(Some((start, end.map(|it| (it + 1).min(size)).unwrap_or(size)))),
        RangeSpec::Suffix(0) => Err(()),
        RangeSpec::Suffix(n) => Ok(Some((size.saturating_sub(n), size))),
    }
}

/// 返回文件数据,文件还没有下载完成时边下载边返回
pub async fn serve(inner: &Arc<Inner>, url: &str, options: &Options, is_head: bool, range: Option<&str>) -> Response<Body> {
    let spec = parse_range(range);
    match inner.downloads.acquire(inner, url, options) {
        Err(e) => error(e),
        Ok(Acquired::Ready(entry, path)) => {
            serve_file(&path, &mime::resolve(entry.content_type.as_deref(), &entry.ext), spec, is_head).await
        }
        Ok(Acquired::Waiting(waiter)) => serve_task(waiter, spec, is_head).await,
    }
}

/// 返回已经下载完成的文件
async fn serve_file(path: &Path, content_type: &str, spec: RangeSpec, is_head: bool) -> Response<Body> {
    let Ok(meta) = tokio::fs::metadata(path).await else {
        return text(StatusCode::NOT_FOUND, "");
    };
    let size = meta.len();
    let Ok(range) = resolve_range(spec, size) else {
        return not_satisfiable(size);
    };
    let (start, end) = range.unwrap_or((0, size));
    let builder = headers(content_type, Some(size), range);
    if is_head {
        return builder.body(empty()).unwrap();
    }
    let Ok(mut file) = File::open(path).await else {
        return text(StatusCode::NOT_FOUND, "");
    };
    if file.seek(SeekFrom::Start(start)).await.is_err() {
        return text(StatusCode::INTERNAL_SERVER_ERROR, "");
    }
    let stream = futures_util::stream::unfold((file, start), move |(mut file, pos)| async move {
        if pos >= end {
            return None;
        }
        let len = (end - pos).min(READ_CHUNK_SIZE) as usize;
        let mut buf = vec![0u8; len];
        match file.read_exact(&mut buf).await {
            Ok(_) => Some((Ok(Bytes::from(buf)), (file, pos + len as u64))),
            Err(e) => Some((Err(e), (file, end))),
        }
    });
    builder.body(stream_body(stream)).unwrap()
}

/// 返回正在下载的文件
async fn serve_task(waiter: Waiter, spec: RangeSpec, is_head: bool) -> Response<Body> {
    let task = waiter.task.clone();
    let ext = task.path.extension().map(|it| it.to_string_lossy().to_string()).unwrap_or_default();
    if let RangeSpec::From(start, _) = spec {
        task.request_pos(start);
    }

    //需要文件大小才能返回响应头,从头读取整个文件时可以不知道文件大小
    let need_size = is_head || !matches!(spec, RangeSpec::All | RangeSpec::From(0, None));
    let mut rx = task.subscribe();
    let data = match rx
        .wait_for(|it| it.result.is_some() || it.size.is_some() || (!need_size && it.ranges.available(0) > 0))
        .await
    {
        Ok(it) => it.clone(),
        Err(_) => return error(SdkError::Cancelled),
    };
    let content_type = mime::resolve(data.content_type.as_deref(), &ext);
    match data.result {
        Some(Ok(path)) => return serve_file(Path::new(&path), &content_type, spec, is_head).await,
        Some(Err(e)) => return error(e),
        None => {}
    }
    let (builder, start, end) = match data.size {
        None => (headers(&content_type, None, None), 0, None),
        Some(size) => {
            let Ok(range) = resolve_range(spec, size) else {
                return not_satisfiable(size);
            };
            let (start, end) = range.unwrap_or((0, size));
            task.request_pos(start);
            (headers(&content_type, Some(size), range), start, Some(end))
        }
    };
    if is_head {
        return builder.body(empty()).unwrap();
    }
    let reader = Reader {
        _waiter: waiter,
        task,
        rx,
        file: None,
        file_pos: 0,
        pos: start,
        end,
        generation: data.generation,
    };
    let stream = futures_util::stream::unfold(reader, |mut reader| async move {
        let chunk = reader.next().await?;
        Some((chunk, reader))
    });
    builder.body(stream_body(stream)).unwrap()
}

/// 读取正在下载的文件,数据还没有下载时等待
struct Reader {
    /// 持有等待,连接断开时释放,没有其他请求时会停止下载
    _waiter: Waiter,
    task: Arc<Task>,
    rx: watch::Receiver<TaskData>,
    file: Option<File>,
    file_pos: u64,
    pos: u64,
    end: Option<u64>,
    generation: u64,
}

impl Reader {
    async fn next(&mut self) -> Option<std::io::Result<Bytes>> {
        loop {
            if self.end.is_some_and(|it| self.pos >= it) {
                return None;
            }
            let (available, size, generation, result) = {
                let data = self.rx.borrow_and_update();
                (data.ranges.available(self.pos), data.size, data.generation, data.result.clone())
            };
            if generation != self.generation {
                return Some(Err(std::io::Error::other("服务器上的文件已经改变")));
            }
            if available > 0 {
                let len = self.end.map(|it| it - self.pos).unwrap_or(u64::MAX).min(available).min(READ_CHUNK_SIZE);
                return Some(self.read(len as usize, result.as_ref().and_then(|it| it.as_ref().ok())).await);
            }
            match result {
                Some(Ok(_)) if size.is_some_and(|it| self.pos >= it) => return None,
                Some(Ok(_)) => return Some(Err(std::io::Error::other("文件数据不完整"))),
                Some(Err(e)) => return Some(Err(e.into())),
                None => {}
            }
            if self.end.is_none() && size.is_some_and(|it| self.pos >= it) {
                return None;
            }

            //数据还没有下载,通知下载任务优先下载这个位置
            self.task.request_pos(self.pos);
            if self.rx.changed().await.is_err() {
                return Some(Err(std::io::Error::other("下载任务已结束")));
            }
        }
    }

    /// 读取数据,下载完成之后临时文件会被改名,这时候从缓存文件读取
    async fn read(&mut self, len: usize, finished: Option<&String>) -> std::io::Result<Bytes> {
        if self.file.is_none() || self.file_pos != self.pos {
            let mut file = match File::open(&self.task.part).await {
                Ok(it) => it,
                Err(e) => match finished {
                    Some(path) => File::open(path).await?,
                    None => return Err(e),
                },
            };
            file.seek(SeekFrom::Start(self.pos)).await?;
            self.file = Some(file);
            self.file_pos = self.pos;
        }
        let mut buf = vec![0u8; len];
        self.file.as_mut().unwrap().read_exact(&mut buf).await?;
        self.pos += len as u64;
        self.file_pos = self.pos;
        Ok(Bytes::from(buf))
    }
}

/// 生成响应头
/// - size 文件大小,未知时不返回Content-Length
/// - range 返回的区间[开始, 结束),None时返回整个文件
fn headers(content_type: &str, size: Option<u64>, range: Option<(u64, u64)>) -> hyper::http::response::Builder {
    let mut builder = Response::builder().header("content-type", content_type).header("cache-control", "no-store");
    let Some(size) = size else {
        return builder;
    };
    builder = builder.header("accept-ranges", "bytes");
    match range {
        Some((start, end)) => builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header("content-range", format!("bytes {}-{}/{}", start, end - 1, size))
            .header("content-length", end - start),
        None => builder.header("content-length", size),
    }
}

fn not_satisfiable(size: u64) -> Response<Body> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header("content-range", format!("bytes */{}", size))
        .body(empty())
        .unwrap()
}

/// 下载失败时的响应,地址过期返回403,文件不存在返回404,其他返回502
fn error(e: SdkError) -> Response<Body> {
    let status = match e {
        SdkError::UrlExpired(_) => StatusCode::FORBIDDEN,
        SdkError::NotFound => StatusCode::NOT_FOUND,
        SdkError::InvalidUrl(_) => StatusCode::BAD_REQUEST,
        SdkError::Cancelled => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::BAD_GATEWAY,
    };
    text(status, &e.to_string())
}

pub fn text(status: StatusCode, msg: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Full::new(Bytes::from(msg.to_string())).map_err(|e| match e {}).boxed())
        .unwrap()
}

fn empty() -> Body {
    Empty::new().map_err(|e| match e {}).boxed()
}

fn stream_body(stream: impl futures_util::Stream<Item = std::io::Result<Bytes>> + Send + Sync + 'static) -> Body {
    BodyExt::boxed(StreamBody::new(stream.map(|it| it.map(Frame::data))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range() {
        assert_eq!(parse_range(None), RangeSpec::All);
        assert_eq!(parse_range(Some("bytes=0-")), RangeSpec::From(0, None));
        assert_eq!(parse_range(Some("bytes=10-19")), RangeSpec::From(10, Some(19)));
        assert_eq!(parse_range(Some("bytes=-5")), RangeSpec::Suffix(5));
        assert_eq!(parse_range(Some("bytes=0-1,5-6")), RangeSpec::All);
        assert_eq!(parse_range(Some("bytes=9-1")), RangeSpec::All);
        assert_eq!(resolve_range(RangeSpec::From(10, Some(19)), 15), Ok(Some((10, 15))));
        assert_eq!(resolve_range(RangeSpec::From(15, None), 15), Err(()));
        assert_eq!(resolve_range(RangeSpec::Suffix(5), 15), Ok(Some((10, 15))));
        assert_eq!(resolve_range(RangeSpec::Suffix(50), 15), Ok(Some((0, 15))));
    }
}
