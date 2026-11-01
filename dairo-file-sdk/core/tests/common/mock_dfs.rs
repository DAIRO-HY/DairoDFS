//! 模拟DFS的上传接口,行为与DFS服务端一致
//! - by_md5:存储里没有这个文件时返回404,已经上传的临时数据保留
//! - by_stream:追加到临时数据,完成后校验MD5,失败时删除临时数据返回500
//! - 同一个MD5同时上传时返回409"文件服务繁忙"

#![allow(dead_code)]

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct DfsState {
    /// 允许的登录票据
    pub tokens: HashSet<String>,

    /// 存储的文件 MD5 => 数据
    pub storage: HashMap<String, Vec<u8>>,

    /// 断点续传的临时数据 MD5 => 数据
    pub temp: HashMap<String, Vec<u8>>,

    /// 正在上传的MD5
    pub uploading: HashSet<String>,

    /// 用户的文件 路径 => MD5
    pub files: HashMap<String, String>,

    /// 收到的请求(路径)
    pub requests: Vec<String>,

    /// 下一次by_stream接收这么多字节之后断开
    pub cut_once: Option<usize>,

    /// by_stream每收到一块数据暂停的毫秒数
    pub slow: u64,

    /// 每次by_stream收到的字节数
    pub stream_received: Vec<usize>,
}

pub struct MockDfs {
    pub base: String,
    pub state: Arc<Mutex<DfsState>>,
}

impl MockDfs {
    pub fn start(token: &str) -> Self {
        let state = Arc::new(Mutex::new(DfsState::default()));
        state.lock().unwrap().tokens.insert(token.to_string());
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
        Self { base: format!("http://127.0.0.1:{}", rx.recv().unwrap()), state }
    }

    pub fn state(&self) -> std::sync::MutexGuard<'_, DfsState> {
        self.state.lock().unwrap()
    }

    /// 收到的指定接口的请求数
    pub fn count(&self, path: &str) -> usize {
        self.state().requests.iter().filter(|it| it.starts_with(path)).count()
    }
}

fn text(status: StatusCode, body: &str) -> Response<Full<Bytes>> {
    Response::builder().status(status).body(Full::new(Bytes::from(body.to_string()))).unwrap()
}

async fn handle(state: Arc<Mutex<DfsState>>, req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let query: HashMap<String, String> = req
        .uri()
        .query()
        .unwrap_or("")
        .split('&')
        .filter_map(|it| it.split_once('='))
        .map(|(k, v)| (k.to_string(), percent_decode(v)))
        .collect();
    let token = req
        .headers()
        .get("authorization")
        .and_then(|it| it.to_str().ok())
        .and_then(|it| it.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string();
    {
        let mut state = state.lock().unwrap();
        state.requests.push(path.clone());
        if !state.tokens.contains(&token) {
            return Ok(text(StatusCode::UNAUTHORIZED, ""));
        }
    }
    let md5 = query.get("md5").cloned().unwrap_or_default();
    match path.as_str() {
        "/api/file_upload/exists" => {
            let state = state.lock().unwrap();
            Ok(text(StatusCode::OK, if state.files.values().any(|it| *it == md5) { "true" } else { "false" }))
        }
        "/api/file_upload/by_md5" => {
            let mut state = state.lock().unwrap();
            if !state.storage.contains_key(&md5) {
                return Ok(text(StatusCode::NOT_FOUND, ""));
            }
            let file_path = query.get("path").cloned().unwrap_or_default();
            if !file_path.starts_with('/') {
                return Ok(text(StatusCode::INTERNAL_SERVER_ERROR, "路径格式不正确"));
            }
            state.files.insert(file_path, md5);
            Ok(text(StatusCode::OK, ""))
        }
        "/api/file_upload/uploaded_size" => {
            let state = state.lock().unwrap();
            if state.uploading.contains(&md5) {
                return Ok(text(StatusCode::CONFLICT, "文件服务繁忙，请稍后重试。"));
            }
            Ok(text(StatusCode::OK, &state.temp.get(&md5).map(|it| it.len()).unwrap_or(0).to_string()))
        }
        _ if path.starts_with("/api/file_upload/by_stream/") => {
            let md5 = path.rsplit('/').next().unwrap().to_string();
            let cut = {
                let mut state = state.lock().unwrap();
                if !state.uploading.insert(md5.clone()) {
                    return Ok(text(StatusCode::CONFLICT, "文件服务繁忙，请稍后重试。"));
                }
                state.cut_once.take()
            };
            let mut body = req.into_body();
            let mut received = 0;
            let mut interrupted = false;
            while let Some(frame) = body.frame().await {
                let Ok(frame) = frame else {
                    interrupted = true;
                    break;
                };
                let Ok(data) = frame.into_data() else {
                    continue;
                };
                let mut data = data.to_vec();
                if let Some(cut) = cut {
                    data.truncate(cut.saturating_sub(received));
                }
                received += data.len();
                state.lock().unwrap().temp.entry(md5.clone()).or_default().extend_from_slice(&data);
                if cut.is_some_and(|it| received >= it) {
                    interrupted = true;
                    break;
                }
                //每次重新读取,测试中途可以修改
                let slow = state.lock().unwrap().slow;
                if slow > 0 {
                    tokio::time::sleep(Duration::from_millis(slow)).await;
                }
            }
            let mut state = state.lock().unwrap();
            state.uploading.remove(&md5);
            state.stream_received.push(received);
            if interrupted {
                return Ok(text(StatusCode::INTERNAL_SERVER_ERROR, "上传中断"));
            }
            let data = state.temp.remove(&md5).unwrap_or_default();
            if format!("{:x}", md5::compute(&data)) != md5 {
                return Ok(text(StatusCode::INTERNAL_SERVER_ERROR, "文件校验失败"));
            }
            state.storage.insert(md5, data);
            Ok(text(StatusCode::OK, ""))
        }
        _ => Ok(text(StatusCode::NOT_FOUND, "")),
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}
