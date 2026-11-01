mod common;

use common::{content, TestServer};
use dairo_file_sdk::{CacheMode, Config, FetchListener, FileSdk, KeyMode, Options, PutSource, SdkError};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 每个测试使用单独的数据目录
fn test_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("dairo-file-sdk-test/{}-{}", name, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sdk(dir: &PathBuf) -> FileSdk {
    sdk_with(dir, |_| {})
}

fn sdk_with(dir: &PathBuf, f: impl FnOnce(&mut Config)) -> FileSdk {
    let mut config = Config::new(dir.to_string_lossy());
    config.retry_times = 2;
    f(&mut config);
    FileSdk::new(config).unwrap()
}

#[derive(Debug)]
#[allow(dead_code)]
enum Event {
    Progress(Option<u64>, u64),
    Success(String),
    Error(SdkError),
}

struct Listener(Mutex<Sender<Event>>);

impl FetchListener for Listener {
    fn on_progress(&self, size: Option<u64>, downloaded: u64) {
        let _ = self.0.lock().unwrap().send(Event::Progress(size, downloaded));
    }
    fn on_success(&self, path: String) {
        let _ = self.0.lock().unwrap().send(Event::Success(path));
    }
    fn on_error(&self, error: SdkError) {
        let _ = self.0.lock().unwrap().send(Event::Error(error));
    }
}

fn listener() -> (Arc<Listener>, Receiver<Event>) {
    let (tx, rx) = channel();
    (Arc::new(Listener(Mutex::new(tx))), rx)
}

/// 等待下载了一部分数据
fn wait_progress(rx: &Receiver<Event>, min: u64) -> u64 {
    loop {
        match rx.recv_timeout(Duration::from_secs(10)).unwrap() {
            Event::Progress(_, downloaded) if downloaded >= min => return downloaded,
            Event::Progress(..) => {}
            other => panic!("下载已经结束:{:?}", other),
        }
    }
}

fn wait_result(rx: &Receiver<Event>) -> Result<String, SdkError> {
    loop {
        match rx.recv_timeout(Duration::from_secs(20)).unwrap() {
            Event::Progress(..) => {}
            Event::Success(path) => return Ok(path),
            Event::Error(e) => return Err(e),
        }
    }
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

#[test]
fn fetch_and_cache() {
    let server = TestServer::start();
    let dir = test_dir("fetch_and_cache");
    let sdk = sdk(&dir);
    let url = server.url("/file/a.mp3?size=100000&token=1");
    let path = sdk.fetch_blocking(&url, Options::default()).unwrap();
    assert!(path.ends_with(".mp3"));
    assert_eq!(read(&path), content(100000, 1));

    //换了票据和域名(命名空间相同)也能命中缓存,不会再请求服务器
    let other = url.replace("token=1", "token=2").replace("127.0.0.1", "localhost");
    let ns = Options { namespace: Some("ns".to_string()), ..Default::default() };
    assert_eq!(sdk.fetch_blocking(&url.replace("token=1", "token=2"), Options::default()).unwrap(), path);
    assert_eq!(sdk.cached_path(&url, &Options::default()).unwrap(), Some(path.clone()));
    assert_eq!(server.received("/file/a.mp3").len(), 1);

    //命名空间不同时是不同的缓存
    let path2 = sdk.fetch_blocking(&other, ns.clone()).unwrap();
    assert_ne!(path, path2);
    assert_eq!(sdk.fetch_blocking(&url, ns).unwrap(), path2);
    assert_eq!(server.received("/file/a.mp3").len(), 2);

    //参数也作为key的一部分
    let query = Options { key_mode: KeyMode::PathAndQuery, ..Default::default() };
    assert_eq!(sdk.cached_path(&url, &query).unwrap(), None);
}

#[test]
fn merge_same_file() {
    let server = TestServer::start();
    let dir = test_dir("merge_same_file");
    let sdk = Arc::new(sdk(&dir));
    let url = server.url("/file/b.bin?size=300000&slow=20");
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let sdk = sdk.clone();
            let url = format!("{}&n={}", url, i);
            std::thread::spawn(move || sdk.fetch_blocking(&url, Options::default()))
        })
        .collect();
    let paths: Vec<String> = handles.into_iter().map(|it| it.join().unwrap().unwrap()).collect();
    assert!(paths.iter().all(|it| *it == paths[0]));
    assert_eq!(read(&paths[0]), content(300000, 1));
    assert_eq!(server.received("/file/b.bin").len(), 1, "同一个文件只下载一次");
}

#[test]
fn cancel_and_resume() {
    let server = TestServer::start();
    let dir = test_dir("cancel_and_resume");
    let sdk = sdk(&dir);
    let url = server.url("/file/c.bin?size=1000000&slow=20");
    let (l, rx) = listener();
    let task = sdk.fetch(&url, Options::default(), l);
    let downloaded = wait_progress(&rx, 100000);
    task.cancel();
    std::thread::sleep(Duration::from_millis(300));
    assert!(downloaded < 1000000);
    assert_eq!(sdk.cached_path(&url, &Options::default()).unwrap(), None);

    let path = sdk.fetch_blocking(&url.replace("slow=20", "slow=0"), Options::default()).unwrap();
    assert_eq!(read(&path), content(1000000, 1));
    let received = server.received("/file/c.bin");
    assert_eq!(received.len(), 2);
    let range = received[1].range.as_deref().unwrap();
    let start: u64 = range.trim_start_matches("bytes=").trim_end_matches('-').parse().unwrap();
    assert!(start >= downloaded, "从上次的位置继续下载:{}", range);
    assert_eq!(received[1].if_range.as_deref(), Some("\"v1\""));
}

#[test]
fn resume_after_restart() {
    let server = TestServer::start();
    let dir = test_dir("resume_after_restart");
    let url = server.url("/file/d.bin?size=8000000&slow=20");
    {
        let sdk = sdk(&dir);
        let (l, rx) = listener();
        let _task = sdk.fetch(&url, Options::default(), l);
        wait_progress(&rx, 1);

        //超过保存进度的间隔(2秒)之后直接退出(模拟App被杀掉)
        std::thread::sleep(Duration::from_millis(3500));
    }
    let sdk = sdk(&dir);
    let path = sdk.fetch_blocking(&url.replace("slow=20", "slow=0"), Options::default()).unwrap();
    assert_eq!(read(&path), content(8000000, 1));
    let received = server.received("/file/d.bin");
    let range = received.last().unwrap().range.as_deref().unwrap();
    assert!(range != "bytes=0-", "重启之后从上次保存的位置继续:{}", range);
}

#[test]
fn content_changed() {
    let server = TestServer::start();
    let dir = test_dir("content_changed");
    let sdk = sdk(&dir);
    let url = server.url("/file/e.bin?size=1000000&slow=20&v=1");
    let (l, rx) = listener();
    let task = sdk.fetch(&url, Options::default(), l);
    wait_progress(&rx, 100000);
    task.cancel();
    std::thread::sleep(Duration::from_millis(300));

    //服务器上的文件已经改变,If-Range不匹配,服务器返回整个新文件
    let path = sdk.fetch_blocking(&server.url("/file/e.bin?size=1200000&v=2"), Options::default()).unwrap();
    assert_eq!(read(&path), content(1200000, 2));
}

#[test]
fn server_without_range() {
    let server = TestServer::start();
    let dir = test_dir("server_without_range");
    let sdk = sdk(&dir);
    let url = server.url("/norange/f.bin?size=1000000&slow=20");
    let (l, rx) = listener();
    let task = sdk.fetch(&url, Options::default(), l);
    wait_progress(&rx, 100000);
    task.cancel();
    std::thread::sleep(Duration::from_millis(300));
    let path = sdk.fetch_blocking(&server.url("/norange/f.bin?size=1000000"), Options::default()).unwrap();
    assert_eq!(read(&path), content(1000000, 1));
}

#[test]
fn unknown_size() {
    let server = TestServer::start();
    let dir = test_dir("unknown_size");
    let sdk = sdk(&dir);
    let path = sdk.fetch_blocking(&server.url("/chunked/g.bin?size=500000"), Options::default()).unwrap();
    assert_eq!(read(&path), content(500000, 1));
}

#[test]
fn retry_after_disconnect() {
    let server = TestServer::start();
    let dir = test_dir("retry_after_disconnect");
    let sdk = sdk(&dir);
    let path = sdk.fetch_blocking(&server.url("/file/h.bin?size=1000000&cut=300000"), Options::default()).unwrap();
    assert_eq!(read(&path), content(1000000, 1));
    let received = server.received("/file/h.bin");
    assert_eq!(received.len(), 2);
    let range = received[1].range.as_deref().unwrap();
    assert!(range != "bytes=0-" && range.starts_with("bytes="), "从断开的位置继续:{}", range);
}

#[test]
fn errors() {
    let server = TestServer::start();
    let dir = test_dir("errors");
    let sdk = sdk_with(&dir, |it| it.retry_times = 1);
    assert_eq!(sdk.fetch_blocking(&server.url("/status/401"), Options::default()), Err(SdkError::UrlExpired(401)));
    assert_eq!(sdk.fetch_blocking(&server.url("/status/404"), Options::default()), Err(SdkError::NotFound));
    let start = Instant::now();
    assert_eq!(sdk.fetch_blocking(&server.url("/status/500"), Options::default()), Err(SdkError::Http(500)));
    assert_eq!(server.received("/status/500").len(), 2, "重试1次");
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert!(matches!(sdk.fetch_blocking("http://127.0.0.1:1/x", Options::default()), Err(SdkError::Network(_))));
    assert!(matches!(sdk.fetch_blocking("abc", Options::default()), Err(SdkError::InvalidUrl(_))));

    //MD5校验
    let url = server.url("/file/i.bin?size=1000");
    let bad = Options { md5: Some("00000000000000000000000000000000".to_string()), ..Default::default() };
    assert_eq!(sdk.fetch_blocking(&url, bad), Err(SdkError::Md5Mismatch));
    let md5 = format!("{:x}", md5::compute(content(1000, 1)));
    let good = Options { md5: Some(md5), ..Default::default() };
    assert_eq!(read(&sdk.fetch_blocking(&url, good).unwrap()), content(1000, 1));
}

#[test]
fn priority_and_concurrency() {
    let server = TestServer::start();
    let dir = test_dir("priority_and_concurrency");
    let sdk = sdk_with(&dir, |it| it.max_caching_count = 1);
    let mut tasks = Vec::new();
    let mut results = Vec::new();
    for (name, priority) in [("p1", 0), ("p2", 0), ("p3", 0), ("p4", 10)] {
        let (l, rx) = listener();
        let options = Options { priority, ..Default::default() };
        tasks.push(sdk.fetch(&server.url(&format!("/file/{}.bin?size=100000&slow=30", name)), options, l));
        results.push(rx);
        std::thread::sleep(Duration::from_millis(50));
    }
    for rx in &results {
        wait_result(rx).unwrap();
    }
    let order: Vec<String> = server.received_all().into_iter().map(|it| it.path).collect();
    assert_eq!(order, ["/file/p1.bin", "/file/p4.bin", "/file/p2.bin", "/file/p3.bin"]);
}

#[test]
fn stop_when_no_waiter() {
    let server = TestServer::start();
    let dir = test_dir("stop_when_no_waiter");
    let sdk = sdk(&dir);
    let url = server.url("/file/j.bin?size=2000000&slow=20");
    let (l1, rx1) = listener();
    let (l2, rx2) = listener();
    let t1 = sdk.fetch(&url, Options::default(), l1);
    let t2 = sdk.fetch(&url, Options::default(), l2);
    wait_progress(&rx1, 100000);

    //还有一个请求在等待,继续下载
    t1.cancel();
    let downloaded = wait_progress(&rx2, 0);
    let more = wait_progress(&rx2, downloaded + 100000);
    assert!(more > downloaded);

    //没有请求在等待,停止下载
    t2.cancel();
    std::thread::sleep(Duration::from_millis(500));
    let size1 = dir_size(&dir.join("files"));
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(size1, dir_size(&dir.join("files")));
    assert!(size1 < 2000000);
    assert_eq!(server.received("/file/j.bin").len(), 1);
}

#[test]
fn save_mode() {
    let server = TestServer::start();
    let dir = test_dir("save_mode");
    let url = server.url("/file/k.bin?size=3000000&slow=20");
    let save = Options { cache: CacheMode::Save, ..Default::default() };
    {
        let sdk = sdk(&dir);
        let (l, rx) = listener();
        let task = sdk.fetch(&url, save.clone(), l);
        wait_progress(&rx, 100000);

        //永久保存的任务取消请求之后也继续下载
        task.cancel();
        std::thread::sleep(Duration::from_millis(2500));
        assert_eq!(server.received("/file/k.bin").len(), 1);
    }

    //重启之后自动继续下载,不需要再次请求
    let sdk = sdk(&dir);
    let start = Instant::now();
    let path = loop {
        if let Some(path) = sdk.cached_path(&url, &Options::default()).unwrap() {
            break path;
        }
        assert!(start.elapsed() < Duration::from_secs(30));
        std::thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(read(&path), content(3000000, 1));
    let received = server.received("/file/k.bin");
    assert_eq!(received.len(), 2);
    assert!(received[1].range.is_some());
}

#[test]
fn temp_mode_removed_after_restart() {
    let server = TestServer::start();
    let dir = test_dir("temp_mode");
    let url = server.url("/file/l.bin?size=1000");
    let temp = Options { cache: CacheMode::None, ..Default::default() };
    let path = {
        let sdk = sdk(&dir);
        let path = sdk.fetch_blocking(&url, temp.clone()).unwrap();
        assert_eq!(sdk.cached_path(&url, &temp).unwrap(), Some(path.clone()));
        path
    };
    let sdk = sdk(&dir);
    assert_eq!(sdk.cached_path(&url, &temp).unwrap(), None);
    assert!(!PathBuf::from(path).exists());
}

#[test]
fn put_file() {
    let server = TestServer::start();
    let dir = test_dir("put_file");
    let sdk = sdk(&dir);
    let url = server.url("/file/m.mov?size=1000");
    let path = sdk.put_file(&url, &Options::default(), PutSource::Bytes(b"hello".to_vec())).unwrap();
    assert!(path.ends_with(".mov"));
    assert_eq!(sdk.fetch_blocking(&url, Options::default()).unwrap(), path);
    assert_eq!(read(&path), b"hello");
    assert!(server.received_all().is_empty());

    //缓存文件被删除之后重新下载
    std::fs::remove_file(&path).unwrap();
    assert_eq!(sdk.cached_path(&url, &Options::default()).unwrap(), None);
    assert_eq!(read(&sdk.fetch_blocking(&url, Options::default()).unwrap()), content(1000, 1));
}

fn dir_size(dir: &PathBuf) -> u64 {
    let mut size = 0;
    for it in std::fs::read_dir(dir).unwrap() {
        let it = it.unwrap();
        if it.file_type().unwrap().is_dir() {
            size += dir_size(&it.path());
        } else {
            size += it.metadata().unwrap().len();
        }
    }
    size
}
