use crate::dao::user_dao;
use crate::extension::string::StringExtension;
use crate::service::{sync, user_service};
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, db, system_config};
use hyperoute_macro::{GET, POST};
use validator::Validate;

/// 获取安装向导的初始数据
#[GET("/api/install/info")]
pub async fn info() -> Result<model::InstallInfo, BizError> {
    check_not_init().await?;
    let config = system_config::get();
    Ok(model::InstallInfo {
        folders: config.save_folder_list.join("\n"),
        sync_domains: config.sync_domains.join("\n"),
    })
}

/// 设置存储目录
#[POST("/api/install/set_storage")]
pub async fn set_storage(form: form::InstallStorage) -> Result<(), BizError> {
    check_not_init().await?;
    let folders = form::split_lines(&form.folders);
    check_folders(&folders)?;
    let mut config = (*system_config::get()).clone();
    config.save_folder_list = folders;
    system_config::save(config)?;
    Ok(())
}

/// 设置分布式同步,设置之后立即从主机全量同步数据
/// - return 是否已经初始化(用户数据已经从主机同步过来时,不需要再创建管理员)
#[POST("/api/install/distributed")]
pub async fn distributed(form: form::InstallDistributed) -> Result<bool, BizError> {
    check_not_init().await?;
    let mut config = (*system_config::get()).clone();
    config.sync_domains = form::split_lines(&form.sync_domains);
    let has_domain = !config.sync_domains.is_empty();
    system_config::save(config)?;
    if !has_domain {
        return Ok(false);
    }
    tokio::spawn(sync::sync_by_table::sync_all());

    //用户表最先同步,稍等用户数据同步过来之后再返回
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if user_dao::count(&mut lib_db::get_context()).await? > 0 {
            return Ok(true);
        }
        if sync::sync_info::has_error() {
            let msg = sync::sync_info::list().iter().map(|it| it.status().msg).collect::<Vec<_>>().join("\n");
            return Err(BizError::msg(msg));
        }
    }
    Ok(false)
}

/// 创建管理员
#[POST("/api/install/create_admin")]
pub async fn create_admin(form: form::CreateAdmin) -> Result<(), BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    let mut ctx = db::begin().await?;

    //在事务里再检查一次,管理员只能被创建一次
    if user_dao::count(&mut ctx).await? > 0 {
        return Err(biz_error!("管理员已经存在,不允许重复创建"));
    }
    user_service::add(&mut ctx, form.name, form.pwd.md5(), None, 1).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 已经初始化之后,不允许再执行安装向导的操作
async fn check_not_init() -> Result<(), BizError> {
    if user_dao::count(&mut lib_db::get_context()).await? > 0 {
        return Err(biz_error!("系统已经初始化,不允许执行该操作"));
    }
    Ok(())
}

/// 检查存储目录是否合法
pub fn check_folders(folders: &[String]) -> Result<(), BizError> {
    if folders.is_empty() {
        return Err(BizError::field("folders", "存储目录不能为空"));
    }
    if folders.iter().any(|it| it.contains("..")) {
        return Err(BizError::field("folders", "目录中不能包含点[..]"));
    }
    Ok(())
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InstallInfo {

        // 存储目录,多个目录用换行分隔
        pub folders: String,

        // 同步服务器URL,多个URL用换行分隔
        pub sync_domains: String,
    }
}
