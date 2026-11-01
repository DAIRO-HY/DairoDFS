use super::Task;
use crate::error::{SdkError, SdkResult};
use crate::index::{Entry, STATE_FINISH};
use crate::event::Event;
use crate::{now_millis, Inner};
use futures_util::StreamExt;
use reqwest::header::{self, HeaderMap};
use reqwest::StatusCode;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// 保存下载进度的间隔
const SAVE_PROGRESS_INTERVAL: Duration = Duration::from_secs(2);

/// 发送下载进度事件的间隔
const PROGRESS_EVENT_INTERVAL: Duration = Duration::from_millis(500);

/// 一次请求的结果
enum Step {
    /// 下载了一部分数据,继续下载剩下的部分
    Continue,

    /// 网络错误,可以重试
    Retry { error: SdkError, progressed: bool },
}

/// 下载文件,完成之后返回缓存文件路径
pub(super) async fn download(inner: &Arc<Inner>, task: &Arc<Task>) -> SdkResult<String> {
    let mut entry = inner.index.get(&task.id)?.ok_or_else(|| SdkError::Db("缓存记录不存在".to_string()))?;
    let path = task.path.clone();
    let part = task.part.clone();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if !tokio::fs::try_exists(&part).await.unwrap_or(false) {
        //临时文件不存在时,之前记录的下载进度无效
        entry.ranges.clear();
    }
    if entry.ranges.is_empty() {
        copy_same_md5(inner, &mut entry, &part).await?;
    }
    let mut file = tokio::fs::OpenOptions::new().create(true).write(true).truncate(false).open(&part).await?;
    task.update(|it| {
        it.started = true;
        it.size = entry.size;
        it.ranges = entry.ranges.clone();
    });
    let mut writer = Writer {
        inner,
        task,
        file: &mut file,
        last_save: Instant::now(),
        last_event: Instant::now(),
        last_event_downloaded: entry.ranges.len(),
    };
    let result = writer.transfer(&mut entry).await;
    writer.save(&mut entry, true).await;
    result?;
    drop(file);

    if let Some(md5) = entry.md5.as_deref().filter(|it| !it.is_empty()) {
        let actual = file_md5(&part).await?;
        if !actual.eq_ignore_ascii_case(md5) {
            let _ = tokio::fs::remove_file(&part).await;
            entry.ranges.clear();
            let _ = inner.index.save_progress(&entry);
            return Err(SdkError::Md5Mismatch);
        }
    }
    tokio::fs::rename(&part, &path).await?;
    let now = now_millis();
    entry.size = Some(entry.ranges.len());
    entry.state = STATE_FINISH;
    entry.error = None;
    entry.finish_date = Some(now);
    entry.access_date = now;
    inner.index.save(&entry)?;
    Ok(path.to_string_lossy().to_string())
}

/// 已经有相同MD5的文件时直接复制,不需要下载
async fn copy_same_md5(inner: &Arc<Inner>, entry: &mut Entry, part: &Path) -> SdkResult<()> {
    let Some(md5) = entry.md5.as_deref().filter(|it| !it.is_empty()) else {
        return Ok(());
    };
    let Some(same) = inner.index.select_finished_by_md5(md5, &entry.id)? else {
        return Ok(());
    };
    if let Ok(size) = tokio::fs::copy(inner.root.join(same.file()), part).await {
        entry.size = Some(size);
        entry.ranges.insert(0, size);
    }
    Ok(())
}

/// 下载中的临时文件路径
pub(crate) fn part_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

struct Writer<'a> {
    inner: &'a Arc<Inner>,
    task: &'a Arc<Task>,
    file: &'a mut File,
    last_save: Instant,

    /// 上次发送进度事件的时间和已经下载的大小,用于计算下载速度
    last_event: Instant,
    last_event_downloaded: u64,
}

impl Writer<'_> {
    /// 循环下载没有下载的部分,直到全部下载完成
    async fn transfer(&mut self, entry: &mut Entry) -> SdkResult<()> {
        let mut fails = 0;
        loop {
            //先判断是否已经下载完成,数据已经完整时即使没有请求在等待也要完成下载
            let Some((start, end)) = entry.ranges.first_missing(self.task.want_pos(), entry.size) else {
                return Ok(());
            };
            if self.task.is_stopped() {
                return Err(SdkError::Cancelled);
            }
            match self.request(entry, start, end).await? {
                Step::Continue => fails = 0,
                Step::Retry { error, progressed } => {
                    fails = if progressed { 1 } else { fails + 1 };
                    if fails > self.inner.config().retry_times {
                        return Err(error);
                    }
                    self.save(entry, true).await;
                    let wait = Duration::from_secs(fails.min(5) as u64);
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {}
                        _ = self.task.stopped() => return Err(SdkError::Cancelled),
                    }
                }
            }
        }
    }

    /// 请求一段数据
    /// - start 开始位置
    /// - end 结束位置(不包含),None表示一直到文件末尾
    async fn request(&mut self, entry: &mut Entry, start: u64, end: Option<u64>) -> SdkResult<Step> {
        let (url, headers) = self.task.request();
        let mut req = self.inner.client.get(&url);
        for (name, value) in &headers {
            req = req.header(name.as_str(), value.as_str());
        }
        let is_range = start > 0 || !entry.ranges.is_empty();
        if is_range {
            let range = match end {
                Some(end) if Some(end) != entry.size => format!("bytes={}-{}", start, end - 1),
                _ => format!("bytes={}-", start),
            };
            req = req.header(header::RANGE, range);

            //文件内容改变时,服务器会返回整个文件
            if let Some(validator) = entry.etag.as_ref().or(entry.last_modified.as_ref()) {
                req = req.header(header::IF_RANGE, validator.as_str());
            }
        }
        let resp = tokio::select! {
            it = req.send() => it,
            _ = self.task.stopped() => return Err(SdkError::Cancelled),
        };
        let resp = match resp {
            Ok(it) => it,
            Err(e) => return Ok(retry(SdkError::Network(e.to_string()), false)),
        };
        let status = resp.status();
        match status {
            StatusCode::PARTIAL_CONTENT => {
                let Some((range_start, total)) = content_range(resp.headers()) else {
                    return Ok(retry(SdkError::Network("服务器返回的Content-Range格式不正确".to_string()), false));
                };
                if range_start != start || (entry.size.is_some() && total.is_some() && entry.size != total) {
                    //文件内容可能已经改变,从头开始下载
                    self.reset(entry).await?;
                    return Ok(Step::Continue);
                }
                entry.size = total.or(entry.size);
                set_validator(entry, resp.headers());
                self.publish(entry);
                self.write(entry, resp, start, end).await
            }
            StatusCode::OK => {
                if !entry.ranges.is_empty() {
                    if is_changed(entry, &resp) {
                        //文件已经改变,从头开始下载
                        self.reset(entry).await?;
                    } else {
                        //服务器不支持断点续传,文件没有改变,从头覆盖写入,已经下载的数据依然有效
                        self.task.set_no_range();
                    }
                }
                entry.size = resp.content_length().or(entry.size);
                set_validator(entry, resp.headers());
                self.publish(entry);
                self.write(entry, resp, 0, None).await
            }
            StatusCode::RANGE_NOT_SATISFIABLE => {
                let total = content_range_total(resp.headers());
                if let Some(total) = total
                    && (entry.size.is_none() || entry.size == Some(total))
                    && start >= total
                {
                    //请求的位置超过了文件大小(例如播放器拖动到了文件末尾之后),知道了文件大小之后从其他位置继续下载
                    entry.size = Some(total);
                    self.publish(entry);
                    return Ok(Step::Continue);
                }
                self.reset(entry).await?;
                Ok(retry(SdkError::Http(status.as_u16()), false))
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(SdkError::UrlExpired(status.as_u16())),
            StatusCode::NOT_FOUND | StatusCode::GONE => Err(SdkError::NotFound),
            _ if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::REQUEST_TIMEOUT => {
                Ok(retry(SdkError::Http(status.as_u16()), false))
            }
            _ => Err(SdkError::Http(status.as_u16())),
        }
    }

    /// 把响应数据写入文件
    async fn write(&mut self, entry: &mut Entry, resp: reqwest::Response, start: u64, end: Option<u64>) -> SdkResult<Step> {
        self.file.seek(SeekFrom::Start(start)).await?;
        self.task.set_segment_start(start);
        let want_pos = self.task.want_pos();
        let mut pos = start;
        let mut stream = resp.bytes_stream();
        loop {
            let chunk = tokio::select! {
                it = stream.next() => it,
                _ = self.task.stopped() => return Err(SdkError::Cancelled),
            };
            let bytes = match chunk {
                Some(Ok(it)) => it,
                Some(Err(e)) => return Ok(retry(SdkError::Network(e.to_string()), pos > start)),
                None => break,
            };
            let mut data = &bytes[..];
            if let Some(end) = end {
                data = &data[..data.len().min((end - pos) as usize)];
            }
            self.file.write_all(data).await?;

            //等待数据真正写入文件之后再通知,正在读取的请求马上就能读到
            self.file.flush().await?;
            entry.ranges.insert(pos, pos + data.len() as u64);
            pos += data.len() as u64;
            self.task.set_current_pos(pos);
            let range = (pos - data.len() as u64, pos);
            self.task.update(|it| it.ranges.insert(range.0, range.1));
            self.save(entry, false).await;
            if entry.size.is_some_and(|it| entry.ranges.is_complete(it)) {
                //数据已经完整,不需要等待服务器关闭连接
                return Ok(Step::Continue);
            }
            if end.is_some_and(|it| pos >= it) {
                //这一段已经下载完成
                return Ok(Step::Continue);
            }
            if self.task.want_pos() != want_pos && !self.task.is_no_range() {
                //需要优先下载其他位置(服务器不支持断点续传时只能顺序下载)
                return Ok(Step::Continue);
            }
        }
        match entry.size {
            //不知道文件大小时,读到结尾就是下载完成
            None => {
                entry.size = Some(pos);
                self.publish(entry);
            }
            Some(size) if pos < end.unwrap_or(size) => {
                return Ok(retry(SdkError::Network("连接被提前关闭".to_string()), pos > start));
            }
            _ => {}
        }
        Ok(Step::Continue)
    }

    /// 清空已经下载的数据
    async fn reset(&mut self, entry: &mut Entry) -> SdkResult<()> {
        self.file.set_len(0).await?;
        entry.ranges.clear();
        entry.size = None;
        entry.etag = None;
        entry.last_modified = None;
        self.task.update(|it| {
            it.ranges.clear();
            it.size = None;
            it.generation += 1;
        });
        self.save(entry, true).await;
        Ok(())
    }

    /// 通知文件大小和类型
    fn publish(&self, entry: &Entry) {
        self.task.update(|it| {
            it.size = entry.size;
            it.content_type = entry.content_type.clone();
        });
    }

    /// 每隔一段时间保存一次下载进度
    /// 保存之前先把数据写入磁盘,确保记录的进度不会超过实际写入的数据
    async fn save(&mut self, entry: &mut Entry, force: bool) {
        let elapsed = self.last_event.elapsed();
        if force || elapsed >= PROGRESS_EVENT_INTERVAL {
            let downloaded = entry.ranges.len();
            let speed = (downloaded.saturating_sub(self.last_event_downloaded) as f64 / elapsed.as_secs_f64().max(0.001)) as u64;
            self.last_event = Instant::now();
            self.last_event_downloaded = downloaded;
            self.inner.events.emit(Event::Progress {
                id: entry.id.clone(),
                mode: self.task.event_mode(),
                size: entry.size,
                downloaded,
                speed,
            });
        }
        if force || self.last_save.elapsed() >= SAVE_PROGRESS_INTERVAL {
            self.last_save = Instant::now();
            if self.file.sync_data().await.is_ok() {
                let _ = self.inner.index.save_progress(entry);
            }
        }
    }
}

fn retry(error: SdkError, progressed: bool) -> Step {
    Step::Retry { error, progressed }
}

/// 服务器返回了整个文件时,判断文件是否已经改变
fn is_changed(entry: &Entry, resp: &reqwest::Response) -> bool {
    let get = |name| resp.headers().get(name).and_then(|it| it.to_str().ok());
    let etag = get(header::ETAG).filter(|it| !it.starts_with("W/"));
    let last_modified = get(header::LAST_MODIFIED);
    (entry.etag.is_some() && entry.etag.as_deref() != etag)
        || (entry.last_modified.is_some() && entry.last_modified.as_deref() != last_modified)
        || (entry.size.is_some() && resp.content_length().is_some() && entry.size != resp.content_length())
}

/// 记录用于确认文件没有改变的标识
fn set_validator(entry: &mut Entry, headers: &HeaderMap) {
    let get = |name| headers.get(name).and_then(|it| it.to_str().ok()).map(str::to_string);

    //弱ETag不能用于If-Range
    entry.etag = get(header::ETAG).filter(|it| !it.starts_with("W/"));
    entry.last_modified = get(header::LAST_MODIFIED);
    entry.content_type = get(header::CONTENT_TYPE);
}

/// 解析Content-Range: bytes 开始-结束/总大小
fn content_range(headers: &HeaderMap) -> Option<(u64, Option<u64>)> {
    let value = headers.get(header::CONTENT_RANGE)?.to_str().ok()?;
    let (range, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, _) = range.split_once('-')?;
    Some((start.trim().parse().ok()?, total.trim().parse().ok()))
}

/// 解析416响应的Content-Range: bytes */总大小
fn content_range_total(headers: &HeaderMap) -> Option<u64> {
    let value = headers.get(header::CONTENT_RANGE)?.to_str().ok()?;
    value.rsplit_once('/')?.1.trim().parse().ok()
}

/// 计算文件的MD5
async fn file_md5(path: &Path) -> SdkResult<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let mut context = md5::Context::new();
        let mut buf = vec![0u8; 1024 * 1024];
        loop {
            let len = file.read(&mut buf)?;
            if len == 0 {
                break;
            }
            context.consume(&buf[..len]);
        }
        Ok(format!("{:x}", context.finalize()))
    })
    .await
    .map_err(|e| SdkError::Io(e.to_string()))?
}
