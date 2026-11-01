mod common;

use common::{content, TestServer};
use dairo_file_sdk::{
    CacheMode, CacheQuery, Config, DownloadState, Event, EventListener, FileSdk, Options, PutSource,
};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    config.retry_times = 1;
    f(&mut config);
    FileSdk::new(config).unwrap()
}

struct Listener(Mutex<Sender<Event>>);

impl EventListener for Listener {
    fn on_event(&self, event: Event) {
        let _ = self.0.lock().unwrap().send(event);
    }
}

fn listen(sdk: &FileSdk) -> Receiver<Event> {
    let (tx, rx) = channel();
    sdk.set_listener(Some(Arc::new(Listener(Mutex::new(tx)))));
    rx
}

/// 等待指定缓存的状态变成state,返回收到的所有事件
fn wait_state(rx: &Receiver<Event>, id: &str, state: DownloadState) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let event = rx.recv_timeout(Duration::from_secs(20)).expect("没有等到状态");
        let done = matches!(&event, Event::StateChanged { id: it, state: s, .. } if it == id && *s == state);
        events.push(event);
        if done {
            return events;
        }
    }
}

fn save() -> Options {
    Options { cache: CacheMode::Save, ..Default::default() }
}

#[test]
fn events_of_fetch() {
    let server = TestServer::start();
    let dir = test_dir("events_of_fetch");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let url = server.url("/file/a.bin?size=2000000&slow=5");
    let id = sdk.cache_id(&url, &Options::default()).unwrap();
    sdk.fetch_blocking(&url, Options::default()).unwrap();
    let events = wait_state(&rx, &id, DownloadState::Finish);
    let states: Vec<DownloadState> = events
        .iter()
        .filter_map(|it| match it {
            Event::StateChanged { state, .. } => Some(*state),
            _ => None,
        })
        .collect();
    assert_eq!(states, [DownloadState::Waiting, DownloadState::Downloading, DownloadState::Finish]);
    let progress: Vec<(u64, u64)> = events
        .iter()
        .filter_map(|it| match it {
            Event::Progress { downloaded, speed, mode, .. } => {
                assert_eq!(*mode, CacheMode::Cache);
                Some((*downloaded, *speed))
            }
            _ => None,
        })
        .collect();
    assert!(progress.len() >= 2, "{:?}", progress);
    assert_eq!(progress.last().unwrap().0, 2000000);
    assert!(progress.iter().any(|(_, speed)| *speed > 0));
}

#[test]
fn download_and_list() {
    let server = TestServer::start();
    let dir = test_dir("download_and_list");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let url = server.url("/file/b.mp4?size=500000");
    let options = Options { name: Some("电影.mp4".to_string()), thumb: Some("http://x/t.jpg".to_string()), ..save() };
    let id = sdk.download(&url, &options).unwrap();
    wait_state(&rx, &id, DownloadState::Finish);

    let info = sdk.get(&id).unwrap().unwrap();
    assert_eq!(info.mode, CacheMode::Save);
    assert_eq!(info.state, DownloadState::Finish);
    assert_eq!((info.size, info.downloaded), (Some(500000), 500000));
    assert_eq!(info.name.as_deref(), Some("电影.mp4"));
    assert_eq!(info.thumb.as_deref(), Some("http://x/t.jpg"));
    assert_eq!(std::fs::read(info.path.unwrap()).unwrap(), content(500000, 1));

    sdk.fetch_blocking(&server.url("/file/c.bin?size=100"), Options::default()).unwrap();
    let saves = sdk.list(&CacheQuery { mode: Some(CacheMode::Save), ..Default::default() }).unwrap();
    assert_eq!(saves.iter().map(|it| it.id.as_str()).collect::<Vec<_>>(), [id.as_str()]);
    assert_eq!(sdk.list(&CacheQuery::default()).unwrap().len(), 2);
    let paged = sdk.list(&CacheQuery { offset: 1, limit: 1, ..Default::default() }).unwrap();
    assert_eq!(paged[0].id, id, "按创建时间倒序");

    //导出到指定文件夹,文件名重复时加序号
    let out = dir.join("out");
    let a = sdk.export(&id, &out.to_string_lossy(), None).unwrap();
    let b = sdk.export(&id, &out.to_string_lossy(), None).unwrap();
    assert!(a.ends_with("电影.mp4") && b.ends_with("电影 (1).mp4"), "{} {}", a, b);
    assert_eq!(std::fs::read(b).unwrap(), content(500000, 1));
}

#[test]
fn pause_and_resume() {
    let server = TestServer::start();
    let dir = test_dir("pause_and_resume");
    let url = server.url("/file/d.bin?size=4000000&slow=20");
    let id = {
        let sdk = sdk(&dir);
        let rx = listen(&sdk);
        let id = sdk.download(&url, &save()).unwrap();
        wait_state(&rx, &id, DownloadState::Downloading);
        std::thread::sleep(Duration::from_millis(500));
        sdk.pause(&[id.clone()]).unwrap();
        wait_state(&rx, &id, DownloadState::Paused);
        let downloaded = sdk.get(&id).unwrap().unwrap().downloaded;
        assert!(downloaded > 0 && downloaded < 4000000);
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(sdk.get(&id).unwrap().unwrap().downloaded, downloaded, "暂停之后不再下载");
        id
    };

    //暂停的任务重启之后保持暂停
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(sdk.get(&id).unwrap().unwrap().state, DownloadState::Paused);
    assert_eq!(server.received("/file/d.bin").len(), 1);

    sdk.resume_all().unwrap();
    wait_state(&rx, &id, DownloadState::Finish);
    let info = sdk.get(&id).unwrap().unwrap();
    assert_eq!(std::fs::read(info.path.unwrap()).unwrap(), content(4000000, 1));
    assert!(server.received("/file/d.bin")[1].range.is_some(), "继续时断点续传");
}

#[test]
fn play_paused_download() {
    let server = TestServer::start();
    let dir = test_dir("play_paused_download");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let url = server.url("/file/p.mp4?size=8000000&slow=20");
    let id = sdk.download(&url, &save()).unwrap();
    wait_state(&rx, &id, DownloadState::Downloading);
    sdk.pause(&[id.clone()]).unwrap();
    wait_state(&rx, &id, DownloadState::Paused);
    let before = sdk.get(&id).unwrap().unwrap().downloaded;

    //播放时只下载需要的部分,状态保持暂停
    let local = sdk.open(&url, &Options::default()).unwrap();
    let data = reqwest::blocking::Client::new()
        .get(&local)
        .header("range", "bytes=5000000-5099999")
        .send()
        .unwrap()
        .bytes()
        .unwrap();
    assert_eq!(data.to_vec(), content(8000000, 1)[5000000..5100000]);
    std::thread::sleep(Duration::from_millis(500));
    let info = sdk.get(&id).unwrap().unwrap();
    assert_eq!(info.state, DownloadState::Paused);
    assert!(info.downloaded > before);
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sdk.get(&id).unwrap().unwrap().downloaded, info.downloaded, "播放结束之后停止下载");
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(event, Event::StateChanged { .. }), "保持暂停,不发送状态改变:{:?}", event);
    }

    //播放中点击继续下载,变成正常的永久保存下载
    let mut resp = reqwest::blocking::get(&local).unwrap();
    let mut buf = vec![0u8; 1000];
    std::io::Read::read_exact(&mut resp, &mut buf).unwrap();
    sdk.resume(&[id.clone()]).unwrap();
    drop(resp);
    wait_state(&rx, &id, DownloadState::Finish);
    let info = sdk.get(&id).unwrap().unwrap();
    assert_eq!(std::fs::read(info.path.unwrap()).unwrap(), content(8000000, 1));
}

#[test]
fn pause_all_pending() {
    let server = TestServer::start();
    let dir = test_dir("pause_all_pending");
    let sdk = sdk_with(&dir, |it| it.max_saving_count = 1);
    let rx = listen(&sdk);
    let ids: Vec<String> =
        (0..3).map(|i| sdk.download(&server.url(&format!("/file/e{}.bin?size=2000000&slow=20", i)), &save()).unwrap()).collect();
    wait_state(&rx, &ids[0], DownloadState::Downloading);

    //正在下载的和排队中的都暂停
    sdk.pause_all().unwrap();
    for id in &ids {
        let start = Instant::now();
        while sdk.get(id).unwrap().unwrap().state != DownloadState::Paused {
            assert!(start.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    std::thread::sleep(Duration::from_millis(300));
    assert!(server.received("/file/e0.bin").len() <= 1);
    assert!(server.received("/file/e1.bin").is_empty() && server.received("/file/e2.bin").is_empty(), "排队中的任务没有开始下载");
}

#[test]
fn delete_while_downloading() {
    let server = TestServer::start();
    let dir = test_dir("delete_while_downloading");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let id = sdk.download(&server.url("/file/f.bin?size=4000000&slow=20"), &save()).unwrap();
    wait_state(&rx, &id, DownloadState::Downloading);
    std::thread::sleep(Duration::from_millis(300));
    sdk.delete(&[id.clone()]).unwrap();
    assert_eq!(sdk.get(&id).unwrap(), None);
    loop {
        if let Event::Removed { ids } = rx.recv_timeout(Duration::from_secs(5)).unwrap() {
            assert_eq!(ids, [id.clone()]);
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(dir_size(&dir.join("files")), 0);
}

#[test]
fn clear_cache_by_group() {
    let server = TestServer::start();
    let dir = test_dir("clear_cache_by_group");
    let sdk = sdk(&dir);
    let music = Options { group: "music".to_string(), ..Default::default() };
    let image = Options { group: "image".to_string(), ..Default::default() };
    sdk.fetch_blocking(&server.url("/file/g1.bin?size=1000"), music.clone()).unwrap();
    sdk.fetch_blocking(&server.url("/file/g2.bin?size=2000"), music.clone()).unwrap();
    sdk.fetch_blocking(&server.url("/file/g3.bin?size=3000"), image.clone()).unwrap();
    sdk.fetch_blocking(&server.url("/file/g4.bin?size=4000"), Options { group: "music".to_string(), ..save() }).unwrap();

    let stats = sdk.stats(Some(CacheMode::Cache)).unwrap();
    let stats: Vec<(String, u64, u64)> = stats.into_iter().map(|it| (it.group, it.count, it.size)).collect();
    assert_eq!(stats, [("image".to_string(), 1, 3000), ("music".to_string(), 2, 3000)]);
    assert_eq!(sdk.stats(None).unwrap().iter().map(|it| it.size).sum::<u64>(), 10000);

    //正在使用的缓存不会被清理
    let playing = sdk.open(&server.url("/file/g5.bin?size=3000000&slow=20"), &music).unwrap();
    let resp = reqwest::blocking::get(&playing).unwrap();

    //永久保存的文件不会被清理
    assert_eq!(sdk.clear_cache(Some("music"), None).unwrap(), 2);
    assert!(sdk.cached_path(&server.url("/file/g3.bin"), &image).unwrap().is_some());
    assert!(sdk.cached_path(&server.url("/file/g4.bin"), &music).unwrap().is_some());
    drop(resp);

    //连接断开之后停止使用,可以被清理
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(sdk.clear_cache(None, None).unwrap(), 2);
}

#[test]
fn auto_clean() {
    let server = TestServer::start();
    let dir = test_dir("auto_clean");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let urls: Vec<String> = (0..4).map(|i| server.url(&format!("/file/h{}.bin?size=100000", i))).collect();
    for url in &urls {
        sdk.fetch_blocking(url, Options::default()).unwrap();
        std::thread::sleep(Duration::from_millis(20));
    }
    let saved = sdk.download(&server.url("/file/h9.bin?size=100000"), &save()).unwrap();
    wait_state(&rx, &saved, DownloadState::Finish);

    //h0被访问过,变成最近访问的
    sdk.cached_path(&urls[0], &Options::default()).unwrap().unwrap();
    sdk.flush();

    //超过容量上限,从最久没有访问的开始删除(h1、h2),永久保存的不算在内
    sdk.set_max_cache_size(250000);
    let start = Instant::now();
    while sdk.cached_path(&urls[2], &Options::default()).unwrap().is_some() {
        assert!(start.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(20));
    }
    let left: Vec<bool> = urls.iter().map(|it| sdk.cached_path(it, &Options::default()).unwrap().is_some()).collect();
    assert_eq!(left, [true, false, false, true]);
    assert!(sdk.get(&saved).unwrap().is_some());

    //超过保存天数(把h3的访问时间改成100天前)
    sdk.set_max_cache_size(0);
    sdk.flush();
    let id = sdk.cache_id(&urls[3], &Options::default()).unwrap();
    let conn = rusqlite::Connection::open(dir.join("index.sqlite")).unwrap();
    let old = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64 - 100 * 86400000;
    conn.execute("update cache set access_date = ?", rusqlite::params![old]).unwrap();
    conn.execute("update cache set access_date = ? where id != ?", rusqlite::params![old + 20 * 86400000, id]).unwrap();
    assert_eq!(sdk.clean().unwrap(), 1);
    assert!(sdk.cached_path(&urls[3], &Options::default()).unwrap().is_none());
    assert!(sdk.cached_path(&urls[0], &Options::default()).unwrap().is_some());
    assert!(sdk.get(&saved).unwrap().is_some(), "永久保存的文件不会过期");
}

#[test]
fn copy_same_md5() {
    let server = TestServer::start();
    let dir = test_dir("copy_same_md5");
    let sdk = sdk(&dir);
    let md5 = format!("{:x}", md5::compute(content(100000, 1)));
    let options = Options { md5: Some(md5.to_uppercase()), ..Default::default() };
    sdk.fetch_blocking(&server.url("/file/i1.bin?size=100000"), options.clone()).unwrap();
    let path = sdk.fetch_blocking(&server.url("/file/i2.bin?size=100000"), Options { md5: Some(md5), ..save() }).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), content(100000, 1));
    assert!(server.received("/file/i2.bin").is_empty(), "相同MD5的文件直接复制");
}

#[test]
fn put_file_events() {
    let server = TestServer::start();
    let dir = test_dir("put_file_events");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let url = server.url("/file/j.bin");
    sdk.put_file(&url, &Options::default(), PutSource::Bytes(b"abc".to_vec())).unwrap();
    let id = sdk.cache_id(&url, &Options::default()).unwrap();
    wait_state(&rx, &id, DownloadState::Finish);
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
