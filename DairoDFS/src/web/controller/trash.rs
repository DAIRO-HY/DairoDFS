use crate::dao::dfs_file_dao;
use crate::service::{dfs_file_delete_service, dfs_file_service};
use crate::util::time_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{db, system_config};
use hyperoute_core::Ext;
use hyperoute_macro::{DELETE, GET, PUT};

/// 获取垃圾箱文件列表
#[GET("/api/trash/list")]
pub async fn list(login: Ext<LoginUser>) -> Result<Vec<model::TrashItem>, BizError> {
    let mut ctx = lib_db::get_context();
    let now = time_util::current_millis();
    let trash_save_time = system_config::get().trash_timeout * 24 * 60 * 60 * 1000;
    let mut list = Vec::new();
    for it in dfs_file_dao::select_delete(&mut ctx, login.id).await? {

        //剩余删除时间
        let time_left = trash_save_time - (now - it.deleteDate.unwrap_or(now));
        let delete_last_time = if time_left > 24 * 60 * 60 * 1000 {
            format!("{}天后删除", time_left / (24 * 60 * 60 * 1000))
        } else if time_left > 60 * 60 * 1000 {
            format!("{}小时后删除", time_left / (60 * 60 * 1000))
        } else if time_left > 60 * 1000 {
            format!("{}分钟后删除", time_left / (60 * 1000))
        } else {
            "即将删除".to_string()
        };
        let has_thumb = it.storageId > 0 && dfs_file_dao::select_extra(&mut ctx, it.id, "thumb").await?.is_some();
        list.push(model::TrashItem {
            id: it.id,
            name: it.name,
            size: it.size,
            date: delete_last_time,
            file_flag: it.storageId > 0,
            has_thumb,
        });
    }
    Ok(list)
}

/// 从垃圾箱还原文件
/// - ids 选中的文件ID
#[PUT("/api/trash/recover")]
pub async fn recover(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_service::trash_recover(&mut ctx, login.id, &ids).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 彻底删除文件
/// - ids 选中的文件ID
#[DELETE("/api/trash/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_delete_service::delete(&mut ctx, login.id, &ids).await?;
    db::commit(ctx).await?;
    Ok(())
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TrashItem {

        // 文件ID
        pub id: i64,

        // 文件名
        pub name: String,

        // 文件大小
        pub size: i64,

        // 剩余删除时间
        pub date: String,

        // 是否文件
        pub file_flag: bool,

        // 是否有缩略图
        pub has_thumb: bool,
    }
}
