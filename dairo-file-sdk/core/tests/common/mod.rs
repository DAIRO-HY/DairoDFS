//! 测试用的HTTP服务器
//!
//! - /file/{name}?size=N&v=版本&slow=毫秒&cut=字节数:支持Range和If-Range,内容和ETag由版本决定;
//!   slow:每发送16KB暂停的毫秒数;cut:第一次请求发送这么多字节之后断开连接
//! - /norange/{name}?size=N:不支持Range,总是返回整个文件
//! - /chunked/{name}?size=N:不返回Content-Length,不支持Range
//! - /status/{code}:返回指定的状态码

#![allow(dead_code)]

pub mod mock_dfs;

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::Frame;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 服务器收到的请求
#[derive(Debug, Clone)]
pub struct Received {
    pub path: String,
    pub range: Option<String>,
    pub if_range: Option<String>,
}

#[derive(Default)]
struct State {
    received: Vec<Received>,

    /// 已经断开过连接的路径
    cut: HashSet<String>,
}

pub struct TestServer {
    pub base: String,
    state: Arc<Mutex<State>>,
}

impl TestServer {
    pub fn start() -> Self {
        let state = Arc::new(Mutex::new(State::default()));
        let (tx, rx) = std::sync::mpsc::channel();
        let server_state = state.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                tx.send(listener.local_addr().unwrap().port()).unwrap();
                loop {
                    let (stream, _) = listener.accept().await.unwrap();
                    let state = server_state.clone();
                    tokio::spawn(async move {
                        let service = service_fn(move |req| handle(state.clone(), req));
                        let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
                    });
                }
            });
        });
        let port = rx.recv().unwrap();
        Self { base: format!("http://127.0.0.1:{}", port), state }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    /// 收到的指定路径的请求
    pub fn received(&self, path: &str) -> Vec<Received> {
        self.state.lock().unwrap().received.iter().filter(|it| it.path == path).cloned().collect()
    }

    pub fn received_all(&self) -> Vec<Received> {
        self.state.lock().unwrap().received.clone()
    }
}

/// 生成测试文件的内容
pub fn content(size: usize, version: u64) -> Vec<u8> {
    (0..size).map(|i| ((i as u64 * 31 + version * 13 + 7) % 251) as u8).collect()
}

type Body = BoxBody<Bytes, Infallible>;

async fn handle(state: Arc<Mutex<State>>, req: Request<hyper::body::Incoming>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path().to_string();
    let query: HashMap<String, String> = req
        .uri()
        .query()
        .unwrap_or("")
        .split('&')
        .filter_map(|it| it.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let header = |name: &str| req.headers().get(name).and_then(|it| it.to_str().ok()).map(str::to_string);
    let range = header("range");
    let if_range = header("if-range");
    state.lock().unwrap().received.push(Received { path: path.clone(), range: range.clone(), if_range: if_range.clone() });

    let param = |name: &str| query.get(name).and_then(|it| it.parse::<u64>().ok());
    let size = param("size").unwrap_or(1000) as usize;
    let version = param("v").unwrap_or(1);
    let slow = param("slow").unwrap_or(0);
    let data = content(size, version);

    if let Some(code) = path.strip_prefix("/status/") {
        let status = StatusCode::from_u16(code.parse().unwrap()).unwrap();
        return Ok(Response::builder().status(status).body(full(Vec::new())).unwrap());
    }
    if path.starts_with("/norange/") {
        return Ok(Response::builder().header("content-length", size).body(slow_body(data, slow, None)).unwrap());
    }
    if path.starts_with("/chunked/") {
        return Ok(Response::builder().body(slow_body(data, slow, None)).unwrap());
    }

    let etag = format!("\"v{}\"", version);
    let cut = param("cut").filter(|_| state.lock().unwrap().cut.insert(path.clone()));
    let builder = Response::builder().header("etag", &etag).header("accept-ranges", "bytes");

    //If-Range不匹配时忽略Range
    let range = range.filter(|_| if_range.as_ref().is_none_or(|it| *it == etag));
    if let Some(range) = range {
        let (start, end) = range.strip_prefix("bytes=").unwrap().split_once('-').unwrap();
        let start: usize = start.parse().unwrap();
        let end: usize = if end.is_empty() { size - 1 } else { end.parse::<usize>().unwrap().min(size - 1) };
        if start >= size {
            return Ok(builder
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header("content-range", format!("bytes */{}", size))
                .body(full(Vec::new()))
                .unwrap());
        }
        return Ok(builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header("content-range", format!("bytes {}-{}/{}", start, end, size))
            .header("content-length", end - start + 1)
            .body(slow_body(data[start..=end].to_vec(), slow, cut))
            .unwrap());
    }
    Ok(builder.header("content-length", size).body(slow_body(data, slow, cut)).unwrap())
}

fn full(data: Vec<u8>) -> Body {
    Full::new(Bytes::from(data)).boxed()
}

/// 每16KB暂停一段时间,cut指定时发送这么多字节之后断开连接
fn slow_body(data: Vec<u8>, slow: u64, cut: Option<u64>) -> Body {
    let chunks: Vec<Bytes> = data.chunks(16 * 1024).map(Bytes::copy_from_slice).collect();
    let stream = futures_util::stream::unfold((chunks.into_iter(), 0u64), move |(mut chunks, sent)| async move {
        if cut.is_some_and(|it| sent >= it) {
            return None;
        }
        let chunk = chunks.next()?;
        if slow > 0 {
            tokio::time::sleep(Duration::from_millis(slow)).await;
        }
        let sent = sent + chunk.len() as u64;
        Some((Ok::<_, Infallible>(Frame::data(chunk)), (chunks, sent)))
    });
    StreamBody::new(stream).boxed()
}
