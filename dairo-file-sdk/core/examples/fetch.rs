//! 手动测试:下载一个文件,可以在下载到指定大小时取消,然后断点续传
//! cargo run -p dairo-file-sdk --example fetch -- <数据目录> <下载地址> [请求头 名称:值] [取消时的大小]
//! 环境变量NAMESPACE:指定命名空间;CANCEL_ONLY=1:取消之后直接退出,不继续下载

use dairo_file_sdk::{Config, FetchListener, FileSdk, Options, SdkError};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

struct Listener(Mutex<Sender<Option<u64>>>);

impl FetchListener for Listener {
    fn on_progress(&self, size: Option<u64>, downloaded: u64) {
        println!("进度:{}/{:?}", downloaded, size);
        let _ = self.0.lock().unwrap().send(Some(downloaded));
    }
    fn on_success(&self, path: String) {
        println!("完成:{}", path);
        let _ = self.0.lock().unwrap().send(None);
    }
    fn on_error(&self, error: SdkError) {
        println!("失败:{}", error);
        let _ = self.0.lock().unwrap().send(None);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sdk = FileSdk::new(Config::new(&args[1])).unwrap();
    let mut options = Options { namespace: std::env::var("NAMESPACE").ok(), ..Default::default() };
    if let Some((name, value)) = args.get(3).and_then(|it| it.split_once(':')) {
        options.headers.push((name.to_string(), value.trim().to_string()));
    }
    if let Some(cancel_at) = args.get(4).and_then(|it| it.parse::<u64>().ok()) {
        let (tx, rx) = channel();
        let task = sdk.fetch(&args[2], options.clone(), Arc::new(Listener(Mutex::new(tx))));
        while let Ok(Some(downloaded)) = rx.recv() {
            if downloaded >= cancel_at {
                task.cancel();
                println!("已取消");
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        if std::env::var("CANCEL_ONLY").is_ok() {
            return;
        }
    }
    let path = sdk.fetch_blocking(&args[2], options).unwrap();
    let data = std::fs::read(&path).unwrap();
    println!("文件:{} 大小:{} MD5:{:x}", path, data.len(), md5::compute(&data));
}
