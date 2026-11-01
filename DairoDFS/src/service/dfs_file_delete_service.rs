use crate::biz_error;
use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::{dfs_file_dao, dfs_file_delete_dao};
use crate::service::dfs_file_service;
use crate::util::time_util;
use crate::web::biz_error::BizError;
use sqlx_context::DbContext;

/// 从垃圾箱彻底删除文件
/// 文件记录移到dfs_file_delete表,本地存储文件由定时任务回收
/// - ids 垃圾箱里的文件ID
pub async fn delete(ctx: &mut DbContext, user_id: i64, ids: &[i64]) -> Result<(), BizError> {
    for id in ids {
        if dfs_file_dao::select_one(&mut *ctx, *id).await.is_err() {
            //已经跟随前面删除的文件夹一起被删除了
            continue;
        }
        let file = dfs_file_service::select_own(ctx, user_id, *id).await?;
        if file.deleteDate.is_none() {
            //该文件未标记为删除
            return Err(biz_error!("非法操作"));
        }
        purge(ctx, file).await?;
    }
    Ok(())
}

/// 彻底删除垃圾箱里超时的文件(定时任务用,不区分用户)
pub async fn delete_by_ids(ctx: &mut DbContext, ids: &[i64]) -> Result<(), BizError> {
    for id in ids {
        let Ok(file) = dfs_file_dao::select_one(&mut *ctx, *id).await else {
            //已经跟随前面删除的文件夹一起被删除了
            continue;
        };
        if file.deleteDate.is_none() {
            continue;
        }
        purge(ctx, file).await?;
    }
    Ok(())
}

/// 彻底删除同一批被删除的记录,包括文件的历史版本
async fn purge(ctx: &mut DbContext, file: DfsFile) -> Result<(), BizError> {
    let delete_date = file.deleteDate.unwrap_or(0);
    for it in dfs_file_dao::select_same_delete(&mut *ctx, file.userId, file.parentId, &file.name, delete_date).await? {
        delete_all(ctx, it).await?;
    }
    Ok(())
}

/// 彻底删除文件,文件夹时删除所有子文件(包括历史版本和已删除的文件)
async fn delete_all(ctx: &mut DbContext, file: DfsFile) -> Result<(), BizError> {
    let now = time_util::current_millis();

    //待删除的文件夹,文件夹太深时递归可能导致栈溢出,所以用循环
    let mut folders = Vec::new();
    if file.storageId > 0 {
        delete_self_and_extra(ctx, &file, now).await?;
    } else {
        folders.push(file);
    }
    while let Some(folder) = folders.pop() {
        for it in dfs_file_dao::select_all_child_list(&mut *ctx, folder.id).await? {
            if it.storageId > 0 {
                delete_self_and_extra(ctx, &it, now).await?;
            } else {
                folders.push(it);
            }
        }
        dfs_file_dao::delete(&mut *ctx, folder.id).await?;
    }
    Ok(())
}

/// 删除文件本身和附属文件
async fn delete_self_and_extra(ctx: &mut DbContext, file: &DfsFile, now: i64) -> Result<(), BizError> {
    if file.isExtra == 0 {
        for it in dfs_file_dao::select_extra_list(&mut *ctx, file.id).await? {
            add_delete(ctx, it.id, now).await?;
        }
    }
    add_delete(ctx, file.id, now).await
}

/// 将文件记录移到彻底删除表
async fn add_delete(ctx: &mut DbContext, id: i64, now: i64) -> Result<(), BizError> {
    dfs_file_delete_dao::insert_from_dfs_file(&mut *ctx, id, now).await?;
    dfs_file_dao::delete(&mut *ctx, id).await?;
    Ok(())
}
