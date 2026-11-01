//! 本机HTTP服务,给播放器、图片组件等只支持URL的组件使用
//!
//! 只监听127.0.0.1,地址里带有随机票据,防止其他App访问
//! - /{票据}/s/{登记ID}/{文件名}:先通过open()登记下载地址和参数,地址里不会带有请求头等敏感信息
//! - /{票据}/c/{参数}/{下载地址}/{文件名}:无状态地址,参数和下载地址都是base64url编码,平台可以自己拼接

mod serve;

use crate::error::{SdkError, SdkResult};
use crate::options::{CacheMode, KeyMode, Options};
use crate::Inner;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasher, Hasher};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::AbortHandle;

/// 最多保存的登记数,超过之后删除最早登记的
const MAX_HANDLE_COUNT: usize = 10000;

/// 连续出错这么多次之后重新监听
const MAX_ACCEPT_ERROR_COUNT: u32 = 10;

/// 无状态地址里的参数(字段名尽量短,减小地址长度)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UrlOptions {
    /// 缓存方式 0:不缓存 1:缓存 2:永久保存
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c: Option<u8>,

    /// 缓存分组
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub g: Option<String>,

    /// 命名空间
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,

    /// 生成缓存key的方式 0:路径 1:路径和参数 2:完整地址
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m: Option<u8>,

    /// 自定义缓存key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub k: Option<String>,

    /// 请求头
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub h: Vec<(String, String)>,

    /// 优先级
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p: Option<i32>,

    /// 文件名
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 文件MD5
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

impl UrlOptions {
    fn from_options(it: &Options) -> Self {
        let default = Options::default();
        Self {
            c: (it.cache != default.cache).then_some(it.cache as u8),
            g: (it.group != default.group).then(|| it.group.clone()),
            n: it.namespace.clone(),
            m: match it.key_mode {
                KeyMode::Path => None,
                KeyMode::PathAndQuery => Some(1),
                KeyMode::Url => Some(2),
            },
            k: it.cache_key.clone(),
            h: it.headers.clone(),
            p: (it.priority != 0).then_some(it.priority),
            name: it.name.clone(),
            md5: it.md5.clone(),
        }
    }

    fn into_options(self) -> Options {
        let default = Options::default();
        Options {
            cache: match self.c {
                Some(0) => CacheMode::None,
                Some(2) => CacheMode::Save,
                _ => CacheMode::Cache,
            },
            group: self.g.unwrap_or(default.group),
            namespace: self.n,
            key_mode: match self.m {
                Some(1) => KeyMode::PathAndQuery,
                Some(2) => KeyMode::Url,
                _ => KeyMode::Path,
            },
            cache_key: self.k,
            headers: self.h,
            priority: self.p.unwrap_or(0),
            name: self.name,
            md5: self.md5,
            thumb: None,
        }
    }
}

/// 正在运行的服务
struct Running {
    port: u16,
    abort: AbortHandle,
}

#[derive(Default)]
struct Handles {
    map: HashMap<String, (String, Options)>,
    order: VecDeque<String>,
}

pub(crate) struct Server {
    /// 访问票据,SDK运行期间不变,服务重启之后之前的地址依然有效
    token: String,

    running: Mutex<Option<Running>>,

    /// 上次监听的端口,重启服务时优先使用
    last_port: Mutex<u16>,

    handles: Mutex<Handles>,
    handle_no: AtomicU64,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            token: random_hex(),
            running: Mutex::new(None),
            last_port: Mutex::new(0),
            handles: Mutex::new(Handles::default()),
            handle_no: AtomicU64::new(0),
        }
    }
}

impl Server {
    /// 登记一个下载地址,返回本机访问地址
    pub fn open(&self, inner: &Arc<Inner>, url: &str, options: &Options) -> SdkResult<String> {
        crate::key::resolve(url, options)?;
        let id = format!("{:x}", self.handle_no.fetch_add(1, Ordering::Relaxed));
        {
            let mut handles = self.handles.lock().unwrap();
            handles.map.insert(id.clone(), (url.to_string(), options.clone()));
            handles.order.push_back(id.clone());
            while handles.order.len() > MAX_HANDLE_COUNT {
                if let Some(old) = handles.order.pop_front() {
                    handles.map.remove(&old);
                }
            }
        }
        Ok(format!("{}/s/{}/{}", self.base(inner)?, id, file_name(url, options)))
    }

    /// 生成无状态的本机访问地址
    pub fn local_url(&self, inner: &Arc<Inner>, url: &str, options: &Options) -> SdkResult<String> {
        crate::key::resolve(url, options)?;
        let params = serde_json::to_vec(&UrlOptions::from_options(options)).unwrap();
        Ok(format!(
            "{}/c/{}/{}/{}",
            self.base(inner)?,
            URL_SAFE_NO_PAD.encode(params),
            URL_SAFE_NO_PAD.encode(url),
            file_name(url, options)
        ))
    }

    /// 本机访问地址的前缀,例如:http://127.0.0.1:12345/票据
    pub fn base(&self, inner: &Arc<Inner>) -> SdkResult<String> {
        let port = self.ensure(inner)?;
        Ok(format!("http://127.0.0.1:{}/{}", port, self.token))
    }

    /// 检查服务是否还能访问,不能访问时重新启动(iOS从后台回到前台时,监听的端口可能已经失效)
    pub fn check(&self, inner: &Arc<Inner>) -> SdkResult<u16> {
        let port = self.running.lock().unwrap().as_ref().map(|it| it.port);
        if let Some(port) = port {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
                return Ok(port);
            }
            self.stop();
        }
        self.ensure(inner)
    }

    /// 停止服务,已经建立的连接不受影响
    pub fn stop(&self) {
        if let Some(it) = self.running.lock().unwrap().take() {
            it.abort.abort();
        }
    }

    /// 服务没有运行时启动服务
    fn ensure(&self, inner: &Arc<Inner>) -> SdkResult<u16> {
        let mut running = self.running.lock().unwrap();
        if let Some(it) = running.as_ref() {
            return Ok(it.port);
        }
        let mut last_port = self.last_port.lock().unwrap();
        let listener = {
            //调用方可能不在tokio的线程里,创建tokio的监听需要进入运行时
            let _guard = inner.rt.enter();
            bind(*last_port)?
        };
        let port = listener.local_addr()?.port();
        *last_port = port;
        let weak = Arc::downgrade(inner);
        let handle = inner.rt.spawn(accept_loop(weak, listener));
        *running = Some(Running { port, abort: handle.abort_handle() });
        Ok(port)
    }

    fn find(&self, id: &str) -> Option<(String, Options)> {
        self.handles.lock().unwrap().map.get(id).cloned()
    }
}

/// 监听端口,优先使用上次的端口,上次的端口被占用时随机选一个
fn bind(port: u16) -> std::io::Result<TcpListener> {
    let listener = if port > 0 {
        std::net::TcpListener::bind(("127.0.0.1", port)).or_else(|_| std::net::TcpListener::bind(("127.0.0.1", 0)))?
    } else {
        std::net::TcpListener::bind(("127.0.0.1", 0))?
    };
    listener.set_nonblocking(true)?;
    TcpListener::from_std(listener)
}

async fn accept_loop(inner: std::sync::Weak<Inner>, listener: TcpListener) {
    let mut errors = 0;
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => {
                errors = 0;
                stream
            }
            Err(e) => {
                errors += 1;
                eprintln!("-->dairo-file-sdk:本机服务接收连接失败:{}", e);
                if errors >= MAX_ACCEPT_ERROR_COUNT {
                    //监听可能已经失效,标记为停止,下次访问时重新启动
                    if let Some(inner) = inner.upgrade() {
                        inner.server.running.lock().unwrap().take();
                    }
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let Some(inner) = inner.upgrade() else {
            return;
        };
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let inner = inner.clone();
                async move { Ok::<_, std::convert::Infallible>(route(&inner, req).await) }
            });
            let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
        });
    }
}

/// 解析请求地址
async fn route(inner: &Arc<Inner>, req: hyper::Request<hyper::body::Incoming>) -> hyper::Response<serve::Body> {
    let is_head = req.method() == hyper::Method::HEAD;
    if req.method() != hyper::Method::GET && !is_head {
        return serve::text(hyper::StatusCode::METHOD_NOT_ALLOWED, "");
    }
    let segments: Vec<&str> = req.uri().path().split('/').collect();
    if segments.len() < 4 || segments[1] != inner.server.token {
        return serve::text(hyper::StatusCode::NOT_FOUND, "");
    }
    let target = match segments[2] {
        "s" => inner.server.find(segments[3]),
        "c" if segments.len() >= 5 => parse_stateless(segments[3], segments[4]),
        _ => None,
    };
    let Some((url, options)) = target else {
        return serve::text(hyper::StatusCode::NOT_FOUND, "");
    };
    let range = req.headers().get(hyper::header::RANGE).and_then(|it| it.to_str().ok()).map(str::to_string);
    serve::serve(inner, &url, &options, is_head, range.as_deref()).await
}

fn parse_stateless(params: &str, url: &str) -> Option<(String, Options)> {
    let params: UrlOptions = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(params).ok()?).ok()?;
    let url = String::from_utf8(URL_SAFE_NO_PAD.decode(url).ok()?).ok()?;
    Some((url, params.into_options()))
}

/// 地址最后的文件名,部分播放器根据扩展名判断文件类型
fn file_name(url: &str, options: &Options) -> String {
    let name = options.name.clone().unwrap_or_else(|| {
        let path = url.split(['?', '#']).next().unwrap_or("");
        path.rsplit('/').next().unwrap_or("").to_string()
    });
    let name = if name.is_empty() { "file".to_string() } else { name };
    let mut encoded = String::new();
    for b in name.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{:02X}", b));
        }
    }
    encoded
}

/// 生成随机票据
fn random_hex() -> String {
    let random = || {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        hasher.finish()
    };
    format!("{:016x}{:016x}", random(), random())
}

impl From<SdkError> for std::io::Error {
    fn from(e: SdkError) -> Self {
        std::io::Error::other(e.to_string())
    }
}
