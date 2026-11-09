mod common;

use common::{content, TestServer};
use dairo_file_sdk::{Config, FileSdk, Options};
use std::path::PathBuf;
use std::time::Duration;

fn sdk(name: &str) -> FileSdk {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir: PathBuf = std::env::temp_dir().join(format!("dairo-file-sdk-test/{}-{}", name, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    FileSdk::new(Config::new(dir.to_string_lossy())).unwrap()
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build().unwrap()
}

fn port_of(url: &str) -> u16 {
    url.trim_start_matches("http://127.0.0.1:").split('/').next().unwrap().parse().unwrap()
}

#[test]
fn ping() {
    let sdk = sdk("debug_ping");
    let base = sdk.local_base().unwrap();
    let resp = client().get(format!("{}/ping", base)).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "application/json; charset=utf-8");
    let json: serde_json::Value = serde_json::from_slice(&resp.bytes().unwrap()).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["port"].as_u64(), Some(port_of(&base) as u64));
    assert!(json["startTime"].as_i64().unwrap() > 0);
    assert!(json["requestCount"].as_u64().unwrap() >= 1);

    //没有票据或者票据不正确时返回404
    let port = port_of(&base);
    for path in ["/ping", "/0123456789abcdef0123456789abcdef/ping", "/x"] {
        let resp = client().get(format!("http://127.0.0.1:{}{}", port, path)).send().unwrap();
        assert_eq!(resp.status(), 404, "{}", path);
    }
    //ping后面带其他路径时也是404
    let resp = client().get(format!("{}/ping/x", base)).send().unwrap();
    assert_eq!(resp.status(), 404);
}

#[test]
fn restart_keeps_port_and_urls() {
    let server = TestServer::start();
    let sdk = sdk("debug_restart");
    let remote = server.url("/file/r.bin?size=100000");
    let url = sdk.local_url(&remote, &Options::default()).unwrap();
    let port = port_of(&url);
    assert_eq!(client().get(&url).send().unwrap().bytes().unwrap().to_vec(), content(100000, 1));

    //刚请求过,原来的连接还处于TIME_WAIT等状态,重启之后依然使用原来的端口
    for _ in 0..3 {
        let new_port = sdk.restart_server().unwrap();
        assert_eq!(new_port, port);
        let resp = client().get(&url).send().unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.bytes().unwrap().to_vec(), content(100000, 1));
    }
    let info = sdk.debug_info();
    assert_eq!(info.server_restart_count, 3);
    assert_eq!(info.server_start_count, 4);
    assert!(info.server_running);
    assert_eq!(info.server_port, Some(port));
    let events: Vec<&str> = info.events.iter().map(|it| it.msg.as_str()).collect();
    assert_eq!(events.iter().filter(|it| it.starts_with("停止:手动重启")).count(), 3, "{:?}", events);
    assert!(events.iter().all(|it| !it.contains("无法使用")), "{:?}", events);
    assert!(events[0].starts_with("启动:端口"), "按时间倒序:{:?}", events);

    //服务没有运行时重启也能启动
    sdk.stop_server();
    assert!(!sdk.debug_info().server_running);
    assert_eq!(sdk.restart_server().unwrap(), port);
    assert_eq!(client().get(&url).send().unwrap().status(), 200);
}

#[test]
fn restart_when_port_taken() {
    let sdk = sdk("debug_restart_taken");
    let base = sdk.local_base().unwrap();
    let port = port_of(&base);
    sdk.stop_server();

    //原来的端口被其他程序占用,换一个端口,并记录事件
    let other = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap();
    let new_port = sdk.restart_server().unwrap();
    assert_ne!(new_port, port);
    let info = sdk.debug_info();
    assert!(info.events[0].msg.contains(&format!("原端口{}无法使用,改用端口{}", port, new_port)), "{:?}", info.events);
    drop(other);
    let base = sdk.local_base().unwrap();
    assert_eq!(client().get(format!("{}/ping", base)).send().unwrap().status(), 200);
}

#[test]
fn self_check() {
    let sdk = sdk("debug_self_check");
    let port = port_of(&sdk.local_base().unwrap());
    let result = sdk.self_check();
    assert!(result.ok, "{:?}", result);
    assert!(result.running && result.connected);
    assert_eq!(result.port, Some(port));
    assert_eq!(result.status, Some(200));
    assert!(result.error.is_none());

    //异步版本
    let result = tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(sdk.self_check_async());
    assert!(result.ok, "{:?}", result);

    //服务停止之后自检失败,并且不会自动启动
    sdk.stop_server();
    let result = sdk.self_check();
    assert!(!result.ok && !result.running && !result.connected, "{:?}", result);
    assert_eq!(result.port, None);
    assert_eq!(result.error.as_deref(), Some("服务没有运行"));
    let info = sdk.debug_info();
    assert!(!info.server_running);
    assert!(info.events[0].msg.starts_with("自检失败:端口无"), "{:?}", info.events);
    assert!(info.events.iter().any(|it| it.msg.starts_with("自检正常:端口")), "{:?}", info.events);
}

#[test]
fn stats() {
    let server = TestServer::start();
    let sdk = sdk("debug_stats");
    let info = sdk.debug_info();
    assert_eq!(info.server_request_count, 0);
    assert_eq!(info.net_request_count, 0);
    assert!(info.sdk_start_time > 0);

    let url = sdk.local_url(&server.url("/file/s.bin?size=200000"), &Options::default()).unwrap();
    let data = client().get(&url).send().unwrap().bytes().unwrap();
    assert_eq!(data.len(), 200000);
    //已经缓存,再请求一次只读缓存
    std::thread::sleep(Duration::from_millis(300));
    let range = client().get(&url).header("range", "bytes=0-99").send().unwrap();
    assert_eq!(range.status(), 206);
    assert_eq!(range.bytes().unwrap().len(), 100);
    let missing = sdk.local_url(&server.url("/status/404"), &Options::default()).unwrap();
    let status = client().get(&missing).send().unwrap().status();
    assert!(status.is_client_error() || status.is_server_error(), "{}", status);
    let bad = format!("{}/x", sdk.local_base().unwrap().rsplit_once('/').unwrap().0);
    assert_eq!(client().get(&bad).send().unwrap().status(), 404);

    std::thread::sleep(Duration::from_millis(200));
    let info = sdk.debug_info();
    assert_eq!(info.server_request_count, 4, "{:?}", info);
    assert_eq!(info.server_status_2xx, 2, "{:?}", info);
    assert_eq!(info.server_status_4xx + info.server_status_5xx, 2, "{:?}", info);
    assert!(info.server_bytes_sent >= 200100, "{:?}", info);
    assert!(info.server_bytes_sent < 200100 + 1000, "{:?}", info);
    assert_eq!(info.net_bytes_received, 200000, "{:?}", info);
    assert!(info.net_request_count >= 2, "{:?}", info);
    assert!(info.net_error_count >= 1, "{:?}", info);
    assert_eq!(info.server_connection_count, 0, "连接都已经断开:{:?}", info);
    assert_eq!(info.downloading_count, 0, "{:?}", info);
}

#[test]
fn local_source() {
    use dairo_file_sdk::LocalSource;
    let server = TestServer::start();
    let sdk = sdk("debug_local_source");

    //没有缓存时返回本机地址
    let remote = server.url("/file/ls.jpg?size=100000");
    let LocalSource::Http(url) = sdk.local_source(&remote, &Options::default()).unwrap() else {
        panic!("没有缓存时应该返回本机地址");
    };
    assert_eq!(url, sdk.local_url(&remote, &Options::default()).unwrap());

    //正在下载时返回本机地址,不会返回不完整的文件
    let slow = server.url("/file/slow.bin?size=2000000&slow=20");
    let task = sdk.fetch(&slow, Options::default(), std::sync::Arc::new(Ignore));
    std::thread::sleep(Duration::from_millis(300));
    assert!(matches!(sdk.local_source(&slow, &Options::default()).unwrap(), LocalSource::Http(_)));
    task.cancel();

    //下载完成之后返回文件路径
    assert_eq!(client().get(&url).send().unwrap().bytes().unwrap().to_vec(), content(100000, 1));
    let start = std::time::Instant::now();
    let path = loop {
        if let LocalSource::File(path) = sdk.local_source(&remote, &Options::default()).unwrap() {
            break path;
        }
        assert!(start.elapsed() < Duration::from_secs(5), "下载完成之后应该返回文件路径");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(std::fs::read(&path).unwrap(), content(100000, 1));

    //返回文件路径时记录访问时间
    let id = sdk.cache_id(&remote, &Options::default()).unwrap();
    let before = sdk.get(&id).unwrap().unwrap().access_date;
    std::thread::sleep(Duration::from_millis(20));
    sdk.local_source(&remote, &Options::default()).unwrap();
    sdk.flush();
    assert!(sdk.get(&id).unwrap().unwrap().access_date > before);
}

struct Ignore;

impl dairo_file_sdk::FetchListener for Ignore {
    fn on_success(&self, _: String) {}
    fn on_error(&self, _: dairo_file_sdk::SdkError) {}
}
