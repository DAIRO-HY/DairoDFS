// #![allow(warnings)] //忽略所有警告
mod application;
mod dao;
mod db;
mod extension;
mod service;
mod system_config;
mod util;
mod web;

/// 使用mimalloc分配内存,用完的内存会还给系统,所有平台都一样
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// mimalloc默认1秒(大块内存10秒)之后才把用完的内存还给系统,生成大图时内存会一直很高,改为用完立即还
/// 选项编号是mimalloc v3.3.2的(Cargo.toml里固定了版本),升级mimalloc时要核对 c_src/mimalloc/v3/include/mimalloc.h
/// 第一次申请内存时mimalloc就会按当时的选项预留内存(在main之前),所以要在程序加载时设置,见INIT_MIMALLOC
extern "C" fn init_mimalloc() {
    const ARENA_EAGER_COMMIT: i32 = 4;
    const PURGE_DELAY: i32 = 15;
    const ARENA_PURGE_MULT: i32 = 24;
    unsafe {
        //预留的内存不预先提交,用到时才占用,否则还内存时不会真正还给系统
        libmimalloc_sys::mi_option_set(ARENA_EAGER_COMMIT, 0);
        libmimalloc_sys::mi_option_set(PURGE_DELAY, 0);
        libmimalloc_sys::mi_option_set(ARENA_PURGE_MULT, 1);
    }
}

/// 程序加载时(main之前、第一次申请内存之前)执行init_mimalloc
#[used]
#[cfg_attr(any(target_os = "linux", target_os = "android", target_os = "freebsd"), unsafe(link_section = ".init_array"))]
#[cfg_attr(target_vendor = "apple", unsafe(link_section = "__DATA,__mod_init_func"))]
#[cfg_attr(windows, unsafe(link_section = ".CRT$XCU"))]
static INIT_MIMALLOC: extern "C" fn() = init_mimalloc;

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

    //启动操作日志写入和清理任务
    service::operate_log_service::start();

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
