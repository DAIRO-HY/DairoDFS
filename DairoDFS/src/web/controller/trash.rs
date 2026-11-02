use crate::dao::dfs_file_dao;
use crate::dao::dfs_file_dao::DfsFile;
use crate::service::operate_log_service::{self, op};
use crate::service::sub_account_service::{perm, Scope};
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
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut list = Vec::new();
    for it in dfs_file_dao::select_delete(&mut ctx, login.owner_id).await? {
        if scope.perms(&mut ctx, &it).await? & perm::DELETE == 0 {
            //子账户只显示有删除权限的文件
            continue;
        }

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
    let files = check(&mut ctx, &login, &ids, perm::DELETE).await?;
    dfs_file_service::trash_recover(&mut ctx, login.owner_id, &ids).await?;
    db::commit(ctx).await?;
    for (file, path) in files {
        operate_log_service::add_file(&login, op::RECOVER, &file, path, None);
    }
    Ok(())
}

/// 彻底删除文件
/// - ids 选中的文件ID
#[DELETE("/api/trash/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    let files = check(&mut ctx, &login, &ids, perm::PURGE).await?;
    dfs_file_delete_service::delete(&mut ctx, login.owner_id, &ids).await?;
    db::commit(ctx).await?;
    for (file, path) in files {
        operate_log_service::add_file(&login, op::PURGE, &file, path, None);
    }
    Ok(())
}

/// 检查子账户对垃圾箱里文件的权限
/// - return 文件和路径,用于记录日志
async fn check(
    ctx: &mut sqlx_context::DbContext,
    login: &LoginUser,
    ids: &[i64],
    need: i64,
) -> Result<Vec<(DfsFile, String)>, BizError> {
    let mut scope = Scope::load(ctx, login).await?;
    let mut files = Vec::new();
    for id in ids {
        if dfs_file_dao::select_one(&mut *ctx, *id).await.is_err() {
            //已经跟随前面删除的文件夹一起被删除了
            continue;
        }
        let file = dfs_file_service::select_own(ctx, login.owner_id, *id).await?;
        scope.check(ctx, &file, need).await?;
        let path = dfs_file_service::get_path_by_id(ctx, file.id).await?;
        files.push((file, path));
    }
    Ok(files)
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
