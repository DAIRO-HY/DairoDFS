//! 本机HTTP服务,给播放器、图片组件等只支持URL的组件使用
//!
//! 只监听127.0.0.1,地址里带有随机票据,防止其他App访问
//! - /{票据}/s/{登记ID}/{文件名}:先通过open()登记下载地址和参数,地址里不会带有请求头等敏感信息
//! - /{票据}/c/{参数}/{下载地址}/{文件名}:无状态地址,参数和下载地址都是base64url编码,平台可以自己拼接
//! - /{票据}/ping:检查服务是否正常(调试用)

mod serve;

use crate::debug::{SelfCheckResult, Stats};
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
use http_body_util::BodyExt;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// 最多保存的登记数,超过之后删除最早登记的
const MAX_HANDLE_COUNT: usize = 10000;

/// 连续出错这么多次之后重新监听
const MAX_ACCEPT_ERROR_COUNT: u32 = 10;

/// 自检的连接和读写超时时间
const SELF_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// 重启时等待原来的监听关闭的最长时间
const STOP_WAIT_TIME: Duration = Duration::from_secs(1);

/// 生成本机地址时检查服务的间隔(毫秒),间隔内的调用不检查
const AUTO_CHECK_INTERVAL: i64 = 2000;

/// 生成本机地址时检查服务的连接超时时间
const AUTO_CHECK_TIMEOUT: Duration = Duration::from_millis(300);

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
    handle: JoinHandle<()>,

    /// 启动时间(毫秒)
    start_time: i64,
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

    /// 生成本机地址时上次检查服务的时间(毫秒)
    last_check: AtomicI64,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            token: random_hex(),
            running: Mutex::new(None),
            last_port: Mutex::new(0),
            handles: Mutex::new(Handles::default()),
            handle_no: AtomicU64::new(0),
            last_check: AtomicI64::new(0),
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
        self.auto_check(inner);
        let port = self.ensure(inner)?;
        Ok(format!("http://127.0.0.1:{}/{}", port, self.token))
    }

    /// 检查服务是否还能访问,不能访问时重新启动(iOS从后台回到前台时,监听的端口可能已经失效)
    pub fn check(&self, inner: &Arc<Inner>) -> SdkResult<u16> {
        let port = self.running.lock().unwrap().as_ref().map(|it| it.port);
        if let Some(port) = port {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
                inner.stats.event(format!("checkServer:端口{}正常", port));
                return Ok(port);
            }
            self.stop(inner, &format!("checkServer:端口{}连接失败,自动重启", port));
        } else {
            inner.stats.event("checkServer:服务没有运行,启动服务");
        }
        self.ensure(inner)
    }

    /// 生成本机地址时检查服务,监听被系统关闭(iOS挂起之后可能会回收监听的端口)时重新启动
    /// 每2秒最多检查一次,同时调用时只有一个调用检查,其他调用不等待
    fn auto_check(&self, inner: &Arc<Inner>) {
        let now = crate::now_millis();
        let last = self.last_check.load(Ordering::Relaxed);
        if now - last < AUTO_CHECK_INTERVAL {
            return;
        }
        if self.last_check.compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed).is_err() {
            return;
        }
        let Some(port) = self.running.lock().unwrap().as_ref().map(|it| it.port) else {
            return;
        };
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if let Err(e) = TcpStream::connect_timeout(&addr, AUTO_CHECK_TIMEOUT) {
            self.stop(inner, &format!("生成本机地址时检查:端口{}连接失败({}),自动重启", port, e));
        }
    }

    /// 停止服务,已经建立的连接不受影响
    /// - reason 停止的原因,记录到调试事件里
    pub fn stop(&self, inner: &Inner, reason: &str) {
        let Some(it) = self.running.lock().unwrap().take() else {
            return;
        };
        it.handle.abort();
        inner.stats.event(format!("停止:{},端口{}", reason, it.port));

        //等待监听真正关闭(在运行时的线程里异步关闭),否则马上重新启动时原来的端口还被占用,只能换一个端口
        let start = Instant::now();
        while !it.handle.is_finished() && start.elapsed() < STOP_WAIT_TIME {
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// 强制重启服务,优先使用原来的端口
    pub fn restart(&self, inner: &Arc<Inner>) -> SdkResult<u16> {
        Stats::add(&inner.stats.server_restart_count, 1);
        self.stop(inner, "手动重启");
        self.ensure(inner)
    }

    /// 当前监听的端口和启动时间,没有运行时都为None
    pub fn running_info(&self) -> (Option<u16>, Option<i64>) {
        match self.running.lock().unwrap().as_ref() {
            Some(it) => (Some(it.port), Some(it.start_time)),
            None => (None, None),
        }
    }

    /// 自检:向本机服务发送一个ping请求,只检查不修复,服务没有运行时不会启动
    pub fn self_check(&self, inner: &Inner) -> SelfCheckResult {
        let start = Instant::now();
        let mut result = SelfCheckResult::default();
        let port = self.running.lock().unwrap().as_ref().map(|it| it.port);
        result.running = port.is_some();
        result.port = port;
        if let Some(port) = port {
            match self.ping(port, &mut result) {
                Ok(()) => result.ok = result.status == Some(200),
                Err(e) => result.error = Some(e),
            }
        } else {
            result.error = Some("服务没有运行".to_string());
        }
        result.elapsed_ms = start.elapsed().as_millis() as u64;
        let port = port.map(|it| it.to_string()).unwrap_or_else(|| "无".to_string());
        if result.ok {
            inner.stats.event(format!("自检正常:端口{},耗时{}ms", port, result.elapsed_ms));
        } else {
            let status = result.status.map(|it| format!(",状态码{}", it)).unwrap_or_default();
            let error = result.error.as_deref().map(|it| format!(",{}", it)).unwrap_or_default();
            inner.stats.event(format!(
                "自检失败:端口{},TCP{}{}{},耗时{}ms",
                port,
                if result.connected { "已连接" } else { "未连接" },
                status,
                error,
                result.elapsed_ms
            ));
        }
        result
    }

    /// 发送ping请求,收到响应时设置状态码
    fn ping(&self, port: u16, result: &mut SelfCheckResult) -> Result<(), String> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let mut stream = TcpStream::connect_timeout(&addr, SELF_CHECK_TIMEOUT).map_err(|e| format!("连接失败:{}", e))?;
        result.connected = true;
        stream.set_read_timeout(Some(SELF_CHECK_TIMEOUT)).map_err(|e| e.to_string())?;
        stream.set_write_timeout(Some(SELF_CHECK_TIMEOUT)).map_err(|e| e.to_string())?;
        let request = format!("GET /{}/ping HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", self.token, port);
        stream.write_all(request.as_bytes()).map_err(|e| format!("发送请求失败:{}", e))?;

        //只需要状态行
        let mut buf = Vec::new();
        let mut chunk = [0u8; 512];
        while !buf.windows(2).any(|it| it == b"\r\n") {
            let n = stream.read(&mut chunk).map_err(|e| match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => "等待响应超时".to_string(),
                _ => format!("读取响应失败:{}", e),
            })?;
            if n == 0 {
                return Err("连接被关闭,没有收到响应".to_string());
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let line = String::from_utf8_lossy(&buf);
        let status = line.split_whitespace().nth(1).and_then(|it| it.parse().ok());
        result.status = status;
        if status.is_none() {
            return Err(format!("响应格式不正确:{}", line.lines().next().unwrap_or("")));
        }
        Ok(())
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
        Stats::add(&inner.stats.server_start_count, 1);
        if *last_port > 0 && *last_port != port {
            inner.stats.event(format!("启动:原端口{}无法使用,改用端口{}", *last_port, port));
        } else {
            inner.stats.event(format!("启动:端口{}", port));
        }
        *last_port = port;
        let weak = Arc::downgrade(inner);
        let handle = inner.rt.spawn(accept_loop(weak, listener));
        *running = Some(Running { port, handle, start_time: crate::now_millis() });
        Ok(port)
    }

    fn find(&self, id: &str) -> Option<(String, Options)> {
        self.handles.lock().unwrap().map.get(id).cloned()
    }
}

/// 监听端口,优先使用上次的端口,上次的端口被占用时随机选一个
fn bind(port: u16) -> std::io::Result<TcpListener> {
    if port > 0
        && let Ok(it) = bind_port(port)
    {
        return Ok(it);
    }
    bind_port(0)
}

/// 设置了SO_REUSEADDR,重启时原来的连接还处于TIME_WAIT状态也能使用原来的端口(端口还在监听时依然不能使用)
fn bind_port(port: u16) -> std::io::Result<TcpListener> {
    let socket = tokio::net::TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind(SocketAddr::from(([127, 0, 0, 1], port)))?;
    socket.listen(1024)
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
                if let Some(inner) = inner.upgrade() {
                    Stats::add(&inner.stats.accept_error_count, 1);
                    *inner.stats.last_accept_error.lock().unwrap() = Some(e.to_string());
                    inner.stats.event(format!("接收连接失败:{}", e));
                }
                if errors >= MAX_ACCEPT_ERROR_COUNT {
                    //监听可能已经失效,标记为停止,下次访问时重新启动
                    if let Some(inner) = inner.upgrade() {
                        inner.server.running.lock().unwrap().take();
                        inner.stats.event(format!("接收连接连续失败{}次,标记为停止,下次访问时重新启动", errors));
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
            let _connection = ConnectionCount::new(inner.clone());
            let service = service_fn(move |req| {
                let inner = inner.clone();
                async move { Ok::<_, std::convert::Infallible>(handle(&inner, req).await) }
            });
            let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
        });
    }
}

/// 当前连接数,连接断开时减1
struct ConnectionCount(Arc<Inner>);

impl ConnectionCount {
    fn new(inner: Arc<Inner>) -> Self {
        Stats::add(&inner.stats.server_connection_count, 1);
        Self(inner)
    }
}

impl Drop for ConnectionCount {
    fn drop(&mut self) {
        self.0.stats.server_connection_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 处理请求,统计请求数、状态码和返回的数据量
async fn handle(inner: &Arc<Inner>, req: hyper::Request<hyper::body::Incoming>) -> hyper::Response<serve::Body> {
    Stats::add(&inner.stats.server_request_count, 1);
    let resp = route(inner, req).await;
    inner.stats.status(resp.status().as_u16());
    let inner = inner.clone();
    resp.map(move |body| {
        body.map_frame(move |frame| {
            if let Some(data) = frame.data_ref() {
                Stats::add(&inner.stats.server_bytes_sent, data.len() as u64);
            }
            frame
        })
        .boxed()
    })
}

/// 解析请求地址
async fn route(inner: &Arc<Inner>, req: hyper::Request<hyper::body::Incoming>) -> hyper::Response<serve::Body> {
    let is_head = req.method() == hyper::Method::HEAD;
    if req.method() != hyper::Method::GET && !is_head {
        return serve::text(hyper::StatusCode::METHOD_NOT_ALLOWED, "");
    }
    let segments: Vec<&str> = req.uri().path().split('/').collect();
    if segments.len() < 3 || segments[1] != inner.server.token {
        return serve::text(hyper::StatusCode::NOT_FOUND, "");
    }
    if segments.len() == 3 && segments[2] == "ping" {
        return ping(inner);
    }
    if segments.len() < 4 {
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

/// 检查服务是否正常,返回JSON
fn ping(inner: &Inner) -> hyper::Response<serve::Body> {
    let (port, start_time) = inner.server.running_info();
    let body = serde_json::json!({
        "ok": true,
        "port": port,
        "startTime": start_time,
        "requestCount": Stats::get(&inner.stats.server_request_count),
    });
    serve::json(body.to_string())
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

#[cfg(test)]
mod tests {
    use crate::{Config, FileSdk};

    /// 模拟iOS上复现的状态:端口还在监听(能建立TCP连接),但是没有人处理请求
    #[test]
    fn self_check_detects_dead_listener() {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("dairo-file-sdk-test/dead_listener-{}", nanos));
        let sdk = FileSdk::new(Config::new(dir.to_string_lossy())).unwrap();
        let inner = &sdk.inner;
        inner.server.base(inner).unwrap();

        //换成一个只监听、不接收连接的端口
        let dead = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let dead_port = dead.local_addr().unwrap().port();
        inner.server.stop(inner, "测试");
        *inner.server.last_port.lock().unwrap() = dead_port;
        let handle = inner.rt.spawn(std::future::pending::<()>());
        *inner.server.running.lock().unwrap() = Some(super::Running { port: dead_port, handle, start_time: 0 });

        //只测试TCP连接的checkServer认为服务正常,不会重启
        assert_eq!(sdk.check_server().unwrap(), dead_port);

        //自检能发现:TCP连接成功,但是等不到响应
        let result = sdk.self_check();
        assert!(!result.ok && result.running && result.connected, "{:?}", result);
        assert_eq!(result.status, None);
        assert_eq!(result.error.as_deref(), Some("等待响应超时"));
        assert!(result.elapsed_ms >= 2900, "{:?}", result);
        let info = sdk.debug_info();
        assert!(info.events[0].msg.contains("TCP已连接") && info.events[0].msg.contains("等待响应超时"), "{:?}", info.events);

        //强制重启:原来的端口还在监听,换一个端口,之后能正常访问
        let port = sdk.restart_server().unwrap();
        assert_ne!(port, dead_port);
        assert!(sdk.debug_info().events[0].msg.contains(&format!("原端口{}无法使用", dead_port)));
        let result = sdk.self_check();
        assert!(result.ok, "{:?}", result);
        drop(dead);
    }
}

#[cfg(test)]
mod auto_check_tests {
    use crate::{Config, FileSdk, Options};
    use std::sync::atomic::Ordering;

    /// 模拟真机上的状态:监听被系统关闭(连接被拒绝),SDK还认为服务在运行
    #[test]
    fn local_url_restarts_closed_listener() {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("dairo-file-sdk-test/auto_check-{}", nanos));
        let sdk = FileSdk::new(Config::new(dir.to_string_lossy())).unwrap();
        let inner = &sdk.inner;
        let url = sdk.local_url("http://example.com/a.jpg", &Options::default()).unwrap();
        let port = inner.server.running_info().0.unwrap();

        //关闭监听但保留运行状态
        let handle = inner.server.running.lock().unwrap().as_ref().unwrap().handle.abort_handle();
        handle.abort();
        while std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(inner.server.running_info().0, Some(port));

        //2秒内不检查
        sdk.local_url("http://example.com/b.jpg", &Options::default()).unwrap();
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_err());

        //超过2秒之后生成地址时检查,在原端口重启,之前的地址依然有效
        inner.server.last_check.store(0, Ordering::Relaxed);
        let url2 = sdk.local_url("http://example.com/a.jpg", &Options::default()).unwrap();
        assert_eq!(url, url2);
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_ok());
        let events = sdk.debug_info().events;
        assert!(events[1].msg.starts_with(&format!("停止:生成本机地址时检查:端口{}连接失败", port)), "{:?}", events);
        assert_eq!(events[0].msg, format!("启动:端口{}", port));

        //服务正常时检查不记录事件
        inner.server.last_check.store(0, Ordering::Relaxed);
        sdk.local_url("http://example.com/a.jpg", &Options::default()).unwrap();
        assert_eq!(sdk.debug_info().events.len(), events.len());

        //checkServer每次都记录
        assert_eq!(sdk.check_server().unwrap(), port);
        assert_eq!(sdk.debug_info().events[0].msg, format!("checkServer:端口{}正常", port));
    }
}
