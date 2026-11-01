mod common;

use common::content;
use common::mock_dfs::MockDfs;
use dairo_file_sdk::{
    Config, Event, EventListener, FileSdk, UploadItem, UploadQuery, UploadSource, UploadSourceProvider, UploadState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TOKEN: &str = "t1";

fn test_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("dairo-file-sdk-test/{}-{}", name, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sdk(dir: &PathBuf) -> FileSdk {
    let mut config = Config::new(dir.join("sdk").to_string_lossy());
    config.retry_times = 3;
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

/// 等待上传状态变成state,返回收到的所有事件
fn wait_state(rx: &Receiver<Event>, id: i64, state: UploadState) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let Ok(event) = rx.recv_timeout(Duration::from_secs(20)) else {
            let states: Vec<&Event> = events.iter().filter(|it| !matches!(it, Event::UploadProgress { .. })).collect();
            panic!("没有等到状态{:?},收到的状态事件:{:?},最后的事件:{:?}", state, states, events.last());
        };
        let done = matches!(&event, Event::UploadStateChanged { id: it, state: s, .. } if *it == id && *s == state);
        events.push(event);
        if done {
            return events;
        }
    }
}

/// 创建一个测试文件
fn file(dir: &PathBuf, name: &str, size: usize, version: u64) -> (String, String) {
    let data = content(size, version);
    let path = dir.join(name);
    std::fs::write(&path, &data).unwrap();
    (path.to_string_lossy().to_string(), format!("{:x}", md5::compute(&data)))
}

fn item(path: &str, dfs_path: &str) -> UploadItem {
    UploadItem::new(UploadSource::File(path.to_string()), dfs_path)
}

#[test]
fn upload_file() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("upload_file");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    sdk.set_dfs_account("a", &format!("{}/", dfs.base), TOKEN);
    let (path, md5) = file(&dir, "a.mp4", 3000000, 1);
    let id = sdk.add_uploads(&[item(&path, "/视频/a.mp4")]).unwrap()[0];
    let events = wait_state(&rx, id, UploadState::Finish);

    assert_eq!(dfs.state().files.get("/视频/a.mp4"), Some(&md5));
    assert_eq!(dfs.state().storage.get(&md5).map(|it| it.len()), Some(3000000));
    assert!(events.iter().any(|it| matches!(it, Event::UploadProgress { hashing: true, done: 3000000, .. })));
    assert!(events.iter().any(|it| matches!(it, Event::UploadProgress { hashing: false, done: 3000000, .. })));

    let info = sdk.get_upload(id).unwrap().unwrap();
    assert_eq!(info.state, UploadState::Finish);
    assert_eq!((info.size, info.uploaded), (Some(3000000), 3000000));
    assert_eq!(info.md5.as_deref(), Some(md5.as_str()));
    assert_eq!(info.name, "a.mp4");
    assert!(info.finish_date.is_some());

    //秒传:服务器上已经有这个文件,不需要再上传数据
    let id = sdk.add_uploads(&[item(&path, "/视频/b.mp4")]).unwrap()[0];
    wait_state(&rx, id, UploadState::Finish);
    assert_eq!(dfs.state().files.get("/视频/b.mp4"), Some(&md5));
    assert_eq!(dfs.count("/api/file_upload/by_stream"), 1);

    //自己的文件里已经有了,跳过
    let skip = UploadItem { skip_if_exists: true, ..item(&path, "/视频/c.mp4") };
    let id = sdk.add_uploads(&[skip]).unwrap()[0];
    wait_state(&rx, id, UploadState::Skipped);
    assert!(!dfs.state().files.contains_key("/视频/c.mp4"));
}

#[test]
fn resume_after_interrupt() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("resume_after_interrupt");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    dfs.state().cut_once = Some(1000000);
    let (path, md5) = file(&dir, "b.bin", 3000000, 2);
    let id = sdk.add_uploads(&[item(&path, "/b.bin")]).unwrap()[0];
    wait_state(&rx, id, UploadState::Finish);
    assert_eq!(dfs.state().stream_received, [1000000, 2000000], "中断之后从断开的位置继续上传");
    assert_eq!(dfs.state().files.get("/b.bin"), Some(&md5));
}

#[test]
fn pause_resume_and_restart() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("pause_resume_and_restart");
    dfs.state().slow = 20;
    let (path, md5) = file(&dir, "c.bin", 20000000, 3);
    let id = {
        let sdk = sdk(&dir);
        let rx = listen(&sdk);
        sdk.set_dfs_account("a", &dfs.base, TOKEN);
        let id = sdk.add_uploads(&[item(&path, "/c.bin")]).unwrap()[0];
        loop {
            if let Event::UploadProgress { hashing: false, done, .. } = rx.recv_timeout(Duration::from_secs(10)).unwrap()
                && done > 500000
            {
                break;
            }
        }
        sdk.pause_uploads(&[id]).unwrap();
        wait_state(&rx, id, UploadState::Paused);
        id
    };

    //暂停之前已经发出的数据还在网络缓冲区里,模拟服务器恢复正常速度之后会很快处理完,并发现连接已经断开
    dfs.state().slow = 0;
    let start = Instant::now();
    while !dfs.state().uploading.is_empty() {
        let received = dfs.state().temp.values().map(|it| it.len()).sum::<usize>();
        assert!(start.elapsed() < Duration::from_secs(5), "暂停之后连接没有断开,服务器已经收到:{}", received);
        std::thread::sleep(Duration::from_millis(50));
    }
    let received = dfs.state().temp.values().map(|it| it.len()).sum::<usize>();
    assert!(received < 20000000, "暂停之后停止发送");

    //重启之后保持暂停
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(sdk.get_upload(id).unwrap().unwrap().state, UploadState::Paused);

    sdk.resume_all_uploads().unwrap();
    wait_state(&rx, id, UploadState::Finish);
    let received = dfs.state().stream_received.clone();
    assert_eq!(received.len(), 2);
    assert!(received[1] < 20000000 && received.iter().sum::<usize>() == 20000000, "{:?}", received);
    assert_eq!(dfs.state().files.get("/c.bin"), Some(&md5));
}

struct MemoryProvider {
    data: HashMap<String, Vec<u8>>,
    closed: Mutex<Vec<String>>,
}

impl UploadSourceProvider for MemoryProvider {
    fn size(&self, source: String) -> Result<u64, String> {
        self.data.get(&source).map(|it| it.len() as u64).ok_or("不存在".to_string())
    }

    fn read(&self, source: String, offset: u64, len: u64) -> Result<Vec<u8>, String> {
        let data = self.data.get(&source).ok_or("不存在".to_string())?;
        let end = (offset + len).min(data.len() as u64) as usize;
        Ok(data[offset as usize..end].to_vec())
    }

    fn close(&self, source: String) {
        self.closed.lock().unwrap().push(source);
    }
}

#[test]
fn custom_source() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("custom_source");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    let data = content(1500000, 4);
    let provider = Arc::new(MemoryProvider { data: HashMap::from([("asset-1".to_string(), data.clone())]), closed: Mutex::default() });
    sdk.set_upload_source_provider(Some(provider.clone()));

    //平台已经计算过MD5时不需要再计算
    let md5 = format!("{:x}", md5::compute(&data));
    let upload = UploadItem { md5: Some(md5.to_uppercase()), ..UploadItem::new(UploadSource::Custom("asset-1".to_string()), "/相册/1.heic") };
    let id = sdk.add_uploads(&[upload]).unwrap()[0];
    let events = wait_state(&rx, id, UploadState::Finish);
    assert!(!events.iter().any(|it| matches!(it, Event::UploadProgress { hashing: true, .. })));
    assert_eq!(dfs.state().storage.get(&md5), Some(&data));
    assert_eq!(*provider.closed.lock().unwrap(), ["asset-1"]);

    //数据来源不存在
    let id = sdk.add_uploads(&[UploadItem::new(UploadSource::Custom("none".to_string()), "/x")]).unwrap()[0];
    let events = wait_state(&rx, id, UploadState::Error);
    assert!(matches!(events.last(), Some(Event::UploadStateChanged { error: Some(e), .. }) if e.contains("不存在")));
    let id = sdk.add_uploads(&[item("/not/exists", "/y")]).unwrap()[0];
    wait_state(&rx, id, UploadState::Error);
}

#[test]
fn accounts_and_token() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("accounts_and_token");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    assert!(sdk.add_uploads(&[item("/x", "/x")]).is_err(), "没有设置账号");

    //票据过期
    sdk.set_dfs_account("a", &dfs.base, "expired");
    let (path, md5) = file(&dir, "d.bin", 100000, 5);
    let id = sdk.add_uploads(&[item(&path, "/d.bin")]).unwrap()[0];
    let events = wait_state(&rx, id, UploadState::Error);
    assert!(matches!(events.last(), Some(Event::UploadStateChanged { error: Some(e), .. }) if e.contains("401")));

    //切换到其他账号,a的任务不会上传
    sdk.set_dfs_account("b", &dfs.base, TOKEN);
    sdk.resume_uploads(&[id]).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sdk.get_upload(id).unwrap().unwrap().state, UploadState::Waiting);
    assert!(dfs.state().files.is_empty());
    let b = sdk.list_uploads(&UploadQuery { account: Some("b".to_string()), ..Default::default() }).unwrap();
    assert!(b.is_empty());

    //重新登录a之后继续上传
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    wait_state(&rx, id, UploadState::Finish);
    assert_eq!(dfs.state().files.get("/d.bin"), Some(&md5));

    //退出登录之后不再上传
    sdk.clear_dfs_account();
    assert!(sdk.add_uploads(&[item(&path, "/e.bin")]).is_err());
}

#[test]
fn busy_and_list() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("busy_and_list");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    let (path, md5) = file(&dir, "f.bin", 200000, 6);

    //同一个文件正在被其他客户端上传,服务器返回繁忙,稍后重试
    dfs.state().uploading.insert(md5.clone());
    {
        let state = dfs.state.clone();
        let md5 = md5.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1500));
            state.lock().unwrap().uploading.remove(&md5);
        });
    }
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    let start = Instant::now();
    let ids = sdk.add_uploads(&[item(&path, "/f1.bin"), item(&path, "/f2.bin"), item(&path, "/f3.bin")]).unwrap();
    for id in &ids {
        wait_state(&rx, *id, UploadState::Finish);
    }
    assert!(start.elapsed() >= Duration::from_millis(1500));
    assert_eq!(dfs.state().files.len(), 3);

    let list = sdk.list_uploads(&UploadQuery::default()).unwrap();
    assert_eq!(list.iter().map(|it| it.id).collect::<Vec<_>>(), ids);
    let paged = sdk.list_uploads(&UploadQuery { offset: 1, limit: 1, ..Default::default() }).unwrap();
    assert_eq!(paged[0].id, ids[1]);

    sdk.delete_uploads(&[ids[0]]).unwrap();
    assert_eq!(sdk.get_upload(ids[0]).unwrap(), None);
    sdk.clear_finished_uploads(None).unwrap();
    assert!(sdk.list_uploads(&UploadQuery::default()).unwrap().is_empty());
    let removed: Vec<Vec<i64>> = rx
        .try_iter()
        .filter_map(|it| match it {
            Event::UploadRemoved { ids } => Some(ids),
            _ => None,
        })
        .collect();
    assert_eq!(removed, [vec![ids[0]], vec![ids[1], ids[2]]]);
}

#[test]
fn delete_while_uploading() {
    let dfs = MockDfs::start(TOKEN);
    let dir = test_dir("delete_while_uploading");
    let sdk = sdk(&dir);
    let rx = listen(&sdk);
    dfs.state().slow = 20;
    sdk.set_dfs_account("a", &dfs.base, TOKEN);
    let (path, _) = file(&dir, "g.bin", 4000000, 7);
    let id = sdk.add_uploads(&[item(&path, "/g.bin")]).unwrap()[0];
    loop {
        if let Event::UploadProgress { hashing: false, .. } = rx.recv_timeout(Duration::from_secs(10)).unwrap() {
            break;
        }
    }
    let start = Instant::now();
    sdk.delete_uploads(&[id]).unwrap();
    assert!(start.elapsed() < Duration::from_secs(5), "删除时等待上传停止:{:?}", start.elapsed());
    assert_eq!(sdk.get_upload(id).unwrap(), None);
    std::thread::sleep(Duration::from_millis(300));
    assert!(dfs.state().files.is_empty());
}
