use crate::system_config;
use crate::service::sync;
use crate::web::biz_error::BizError;
use crate::web::controller::install;
use crate::web::form;
use hyperoute_macro::{GET, POST, PUT};
use rand::distr::{Alphanumeric, SampleString};
use validator::Validate;

/// 获取系统配置
#[GET("/api/profile/info")]
pub async fn info() -> model::ProfileInfo {
    let config = system_config::get();
    model::ProfileInfo {
        open_sql_log: config.open_sql_log,
        is_read_only: config.is_read_only,
        folders: config.save_folder_list.join("\n"),
        sync_domains: config.sync_domains.join("\n"),
        token: config.distributed_token.clone(),
        trash_timeout: config.trash_timeout,
        delete_storage_timeout: config.delete_storage_timeout,
        thumb_max_size: config.thumb_max_size,
        ignore_sync_error: config.ignore_sync_error,
        db_backup_expire_day: config.db_backup_expire_day,
    }
}

/// 更新系统配置
#[POST("/api/profile/update")]
pub async fn update(form: form::Profile) -> Result<(), BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    let folders = form::split_lines(&form.folders);
    install::check_folders(&folders)?;
    let mut config = (*system_config::get()).clone();
    config.save_folder_list = folders;
    config.open_sql_log = form.open_sql_log;
    config.is_read_only = form.is_read_only;
    config.sync_domains = form::split_lines(&form.sync_domains);
    config.trash_timeout = form.trash_timeout;
    config.delete_storage_timeout = form.delete_storage_timeout;
    config.thumb_max_size = form.thumb_max_size;
    config.ignore_sync_error = form.ignore_sync_error;
    config.db_backup_expire_day = form.db_backup_expire_day;
    system_config::save(config)?;

    //同步主机可能有变化,重新开始监听
    sync::listen_all();
    Ok(())
}

/// 重新生成分布式同步票据
#[PUT("/api/profile/make_token")]
pub async fn make_token() -> Result<(), BizError> {
    let mut config = (*system_config::get()).clone();
    config.distributed_token = Alphanumeric.sample_string(&mut rand::rng(), 32);
    system_config::save(config)?;

    //同步连接里包含本机的票据,需要重新开始监听
    sync::listen_all();
    Ok(())
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ProfileInfo {

        // 记录同步日志
        pub open_sql_log: bool,

        // 将当前服务器设置为只读,仅作为备份使用
        pub is_read_only: bool,

        // 存储目录,多个目录用换行分隔
        pub folders: String,

        // 同步服务器URL,多个URL用换行分隔
        pub sync_domains: String,

        // 分机与主机同步连接票据
        pub token: String,

        // 回收站超时(单位：天)
        pub trash_timeout: i64,

        // 删除没有被使用的文件超时设置(单位：天)
        pub delete_storage_timeout: i64,

        // 缩略图最大边尺寸
        pub thumb_max_size: i64,

        // 忽略本机同步错误
        pub ignore_sync_error: bool,

        // 数据库备份天数
        pub db_backup_expire_day: i64,
    }
}
