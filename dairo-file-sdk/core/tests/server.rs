mod common;

use common::{content, TestServer};
use dairo_file_sdk::{Config, FileSdk, Options};
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn test_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("dairo-file-sdk-test/{}-{}", name, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sdk(name: &str) -> (FileSdk, PathBuf) {
    let dir = test_dir(name);
    let mut config = Config::new(dir.to_string_lossy());
    config.retry_times = 1;
    (FileSdk::new(config).unwrap(), dir)
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().unwrap()
}

fn get(url: &str, range: Option<&str>) -> reqwest::blocking::Response {
    let mut req = client().get(url);
    if let Some(range) = range {
        req = req.header("range", range);
    }
    req.send().unwrap()
}

fn header(resp: &reqwest::blocking::Response, name: &str) -> Option<String> {
    resp.headers().get(name).map(|it| it.to_str().unwrap().to_string())
}

/// 服务器收到的请求的开始位置
fn range_starts(server: &TestServer, path: &str) -> Vec<u64> {
    server
        .received(path)
        .iter()
        .map(|it| {
            it.range
                .as_deref()
                .map(|it| it.trim_start_matches("bytes=").split('-').next().unwrap().parse().unwrap())
                .unwrap_or(0)
        })
        .collect()
}

#[test]
fn full_and_head() {
    let server = TestServer::start();
    let (sdk, _) = sdk("full_and_head");
    let url = sdk.open(&server.url("/file/a.mp3?size=300000"), &Options::default()).unwrap();
    assert!(url.starts_with("http://127.0.0.1:") && url.ends_with("/a.mp3"), "{}", url);

    let head = client().head(&url).send().unwrap();
    assert_eq!(head.status(), 200);
    assert_eq!(header(&head, "content-length").as_deref(), Some("300000"));
    assert_eq!(header(&head, "accept-ranges").as_deref(), Some("bytes"));
    assert_eq!(header(&head, "content-type").as_deref(), Some("audio/mpeg"));

    let resp = get(&url, None);
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.bytes().unwrap().to_vec(), content(300000, 1));

    //已经缓存,直接读取缓存文件
    wait_cached(&sdk, &server.url("/file/a.mp3?size=300000"), &Options::default());
    let resp = get(&url, Some("bytes=100-199"));
    assert_eq!(resp.status(), 206);
    assert_eq!(header(&resp, "content-range").as_deref(), Some("bytes 100-199/300000"));
    assert_eq!(resp.bytes().unwrap().to_vec(), content(300000, 1)[100..200]);
    //HEAD请求结束之后没有请求在等待,下载停止(可能还没有下载到数据);GET请求重新下载或者从停止的位置继续下载;之后读取缓存不再请求
    let received = server.received("/file/a.mp3");
    assert!(received.len() <= 2, "{:?}", received);
}

#[test]
fn seek_before_downloaded() {
    let server = TestServer::start();
    let (sdk, _) = sdk("seek_before_downloaded");

    //按每20毫秒16KB的速度,下载整个文件需要6秒以上
    let url = sdk.open(&server.url("/file/b.mp4?size=5000000&slow=20"), &Options::default()).unwrap();
    let start = Instant::now();
    let resp = get(&url, Some("bytes=4000000-4099999"));
    assert_eq!(resp.status(), 206);
    assert_eq!(header(&resp, "content-range").as_deref(), Some("bytes 4000000-4099999/5000000"));
    assert_eq!(resp.bytes().unwrap().to_vec(), content(5000000, 1)[4000000..4100000]);
    assert!(start.elapsed() < Duration::from_secs(3), "跳到拖动的位置下载:{:?}", start.elapsed());
    assert!(range_starts(&server, "/file/b.mp4").contains(&4000000), "{:?}", server.received("/file/b.mp4"));
}

#[test]
fn stream_while_downloading() {
    let server = TestServer::start();
    let (sdk, _) = sdk("stream_while_downloading");
    let url = sdk.open(&server.url("/file/c.bin?size=1000000&slow=5"), &Options::default()).unwrap();
    let resp = get(&url, None);
    assert_eq!(header(&resp, "content-length").as_deref(), Some("1000000"));
    assert_eq!(resp.bytes().unwrap().to_vec(), content(1000000, 1));
}

#[test]
fn player_two_connections() {
    let server = TestServer::start();
    let (sdk, _) = sdk("player_two_connections");

    //播放器通常先读开头,再读结尾的索引数据(mp4的moov)
    let url = sdk.open(&server.url("/file/d.mp4?size=3000000&slow=10"), &Options::default()).unwrap();
    let head = {
        let url = url.clone();
        std::thread::spawn(move || get(&url, Some("bytes=0-")).bytes().unwrap().to_vec())
    };
    std::thread::sleep(Duration::from_millis(200));
    let tail = get(&url, Some("bytes=-100000")).bytes().unwrap().to_vec();
    let data = content(3000000, 1);
    assert_eq!(tail, data[2900000..]);
    assert_eq!(head.join().unwrap(), data);
    assert_eq!(server.received("/file/d.mp4").iter().filter(|it| it.range.is_none()).count(), 1);
}

#[test]
fn disconnect_stops_download() {
    let server = TestServer::start();
    let (sdk, dir) = sdk("disconnect_stops_download");
    let url = sdk.open(&server.url("/file/e.bin?size=3000000&slow=20"), &Options::default()).unwrap();
    let mut resp = get(&url, None);
    let mut buf = vec![0u8; 100000];
    resp.read_exact(&mut buf).unwrap();
    assert_eq!(buf, content(3000000, 1)[..100000]);
    drop(resp);
    std::thread::sleep(Duration::from_millis(500));
    let size = dir_size(&dir.join("files"));
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(size, dir_size(&dir.join("files")), "连接断开之后停止下载");
    assert!(size < 3000000);
}

#[test]
fn stateless_url() {
    let server = TestServer::start();
    let (sdk, _) = sdk("stateless_url");
    let origin = server.url("/file/f.jpg?size=20000&token=1");
    let options = Options { namespace: Some("ns".to_string()), group: "thumb".to_string(), ..Default::default() };
    let url = sdk.local_url(&origin, &options).unwrap();
    assert!(url.contains("/c/") && url.ends_with("/f.jpg"));
    let resp = get(&url, None);
    assert_eq!(header(&resp, "content-type").as_deref(), Some("image/jpeg"));
    assert_eq!(resp.bytes().unwrap().to_vec(), content(20000, 1));

    //无状态地址和fetch使用同一个缓存(数据返回完之后还要改名,稍等一下)
    let path = wait_cached(&sdk, &origin.replace("token=1", "token=2"), &options);
    assert_eq!(std::fs::read(path).unwrap(), content(20000, 1));
    assert!(sdk.local_base().unwrap().starts_with("http://127.0.0.1:"));

    //票据不正确或者登记不存在
    let base = sdk.local_base().unwrap();
    let wrong = format!("{}/abc/s/0/a", base.rsplit_once('/').unwrap().0);
    assert_eq!(get(&wrong, None).status(), 404);
    assert_eq!(get(&format!("{}/s/zzz/a", base), None).status(), 404);
    assert_eq!(get(&format!("{}/c/xx/yy/a", base), None).status(), 404);
}

#[test]
fn range_not_satisfiable() {
    let server = TestServer::start();
    let (sdk, _) = sdk("range_not_satisfiable");
    let url = sdk.open(&server.url("/file/g.bin?size=1000&slow=50"), &Options::default()).unwrap();
    let resp = get(&url, Some("bytes=2000-"));
    assert_eq!(resp.status(), 416);
    assert_eq!(header(&resp, "content-range").as_deref(), Some("bytes */1000"));
    assert_eq!(get(&url, None).bytes().unwrap().len(), 1000);
    assert_eq!(get(&url, Some("bytes=2000-")).status(), 416);
}

#[test]
fn unknown_size() {
    let server = TestServer::start();
    let (sdk, _) = sdk("unknown_size");
    let url = sdk.open(&server.url("/chunked/h.bin?size=500000&slow=5"), &Options::default()).unwrap();
    let resp = get(&url, None);
    assert_eq!(header(&resp, "content-length"), None);
    assert_eq!(resp.bytes().unwrap().to_vec(), content(500000, 1));

    //下载完成之后知道了文件大小
    let resp = get(&url, Some("bytes=10-19"));
    assert_eq!(resp.status(), 206);
    assert_eq!(resp.bytes().unwrap().to_vec(), content(500000, 1)[10..20]);
}

#[test]
fn errors() {
    let server = TestServer::start();
    let (sdk, _) = sdk("server_errors");
    assert_eq!(get(&sdk.open(&server.url("/status/404"), &Options::default()).unwrap(), None).status(), 404);
    assert_eq!(get(&sdk.open(&server.url("/status/401"), &Options::default()).unwrap(), None).status(), 403);
    assert!(sdk.open("abc", &Options::default()).is_err());
}

#[test]
fn restart_server() {
    let server = TestServer::start();
    let (sdk, _) = sdk("restart_server");
    let url = sdk.open(&server.url("/file/i.bin?size=1000"), &Options::default()).unwrap();
    let port = sdk.check_server().unwrap();
    assert_eq!(get(&url, None).bytes().unwrap().len(), 1000);

    //模拟iOS回到前台时端口已经失效
    sdk.stop_server();
    assert!(client().get(&url).send().is_err());
    assert_eq!(sdk.check_server().unwrap(), port, "重启之后使用原来的端口");
    assert_eq!(get(&url, None).bytes().unwrap().to_vec(), content(1000, 1));
}

fn wait_cached(sdk: &FileSdk, url: &str, options: &Options) -> String {
    let start = Instant::now();
    loop {
        if let Some(path) = sdk.cached_path(url, options).unwrap() {
            return path;
        }
        assert!(start.elapsed() < Duration::from_secs(5), "没有缓存:{}", url);
        std::thread::sleep(Duration::from_millis(20));
    }
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
