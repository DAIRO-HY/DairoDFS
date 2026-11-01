// #![allow(warnings)] //忽略所有警告
mod application;
mod dao;
mod db;
mod extension;
mod service;
mod system_config;
mod util;
mod web;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("{:?}", *application::ARGS);

    //数据保存目录不存在时创建
    if let Err(e) = std::fs::create_dir_all(&application::ARGS.data_dir) {
        eprintln!("-->创建数据保存目录:{}失败: {}", application::ARGS.data_dir, e);
        std::process::exit(1);
    }

    //临时文件目录(上传文件的临时文件也放在这里,和存储目录在同一块磁盘上,移动文件时可以直接改名)
    if let Err(e) = std::fs::create_dir_all(application::TEMP_PATH.as_path()) {
        eprintln!("-->创建临时目录:{}失败: {}", application::TEMP_PATH.display(), e);
        std::process::exit(1);
    }
    hyperoute_core::set_upload_temp_dir(application::TEMP_PATH.as_path());

    //没有通过环境变量设置请求体大小上限时,默认最大允许上传10GB的文件
    if std::env::var_os("HYPEROUTE_MAX_BODY_SIZE").is_none() {
        hyperoute_core::set_max_body_size(Some(10 * 1024 * 1024 * 1024));
    }

    lib_db::init(&application::ARGS.data_dir).await;

    //加载系统配置
    system_config::init();
    web::server::ready();

    //启动文件处理任务(生成缩略图、预览图等)
    service::file_handle_service::start();

    //开始监听同步主机的日志
    service::sync::listen_all();

    //启动回收定时器(每天凌晨3点备份数据库、回收存储空间)
    service::recycle_service::start();

    //等待程序退出
    application::SHUTDOWN_NOTIFY.notified().await;
    println!("-->即将退出");

    // 等待一段时间再退出确保监听端口都已经关闭
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("-->程序已退出");
    Ok(())
}
