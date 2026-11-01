//! 手动测试:上传一个文件到DFS
//! cargo run -p dairo-file-sdk --example upload -- <数据目录> <DFS地址> <登录票据> <本地文件> <DFS路径>

use dairo_file_sdk::{Config, Event, EventListener, FileSdk, UploadItem, UploadSource, UploadState};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

struct Listener(Mutex<Sender<Event>>);

impl EventListener for Listener {
    fn on_event(&self, event: Event) {
        let _ = self.0.lock().unwrap().send(event);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sdk = FileSdk::new(Config::new(&args[1])).unwrap();
    let (tx, rx) = channel();
    sdk.set_listener(Some(Arc::new(Listener(Mutex::new(tx)))));
    sdk.set_dfs_account("test", &args[2], &args[3]);
    let id = sdk.add_uploads(&[UploadItem::new(UploadSource::File(args[4].clone()), &args[5])]).unwrap()[0];
    while let Ok(event) = rx.recv() {
        match event {
            Event::UploadProgress { hashing, size, done, .. } => println!("{}:{}/{}", if hashing { "计算MD5" } else { "上传" }, done, size),
            Event::UploadStateChanged { id: it, state, error } if it == id => {
                println!("状态:{:?} {}", state, error.unwrap_or_default());
                if matches!(state, UploadState::Finish | UploadState::Skipped | UploadState::Error) {
                    break;
                }
            }
            _ => {}
        }
    }
}
