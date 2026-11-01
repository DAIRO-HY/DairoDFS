//! 手动测试:登记一个下载地址,打印本机HTTP地址,用播放器或浏览器访问
//! cargo run -p dairo-file-sdk --example serve -- <数据目录> <下载地址> [请求头 名称:值]
//! 环境变量NAMESPACE:指定命名空间

use dairo_file_sdk::{Config, FileSdk, Options};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sdk = FileSdk::new(Config::new(&args[1])).unwrap();
    let mut options = Options { namespace: std::env::var("NAMESPACE").ok(), ..Default::default() };
    if let Some((name, value)) = args.get(3).and_then(|it| it.split_once(':')) {
        options.headers.push((name.to_string(), value.trim().to_string()));
    }
    println!("{}", sdk.open(&args[2], &options).unwrap());
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
