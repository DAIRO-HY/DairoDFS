use crate::dao::dfs_file_dao;
use crate::dao::dfs_file_dao::{DfsFile, MoveFileWithHistoryParam};
use crate::util::{file_util, id_util, time_util};
use crate::web::biz_error::BizError;
use crate::{biz_error, biz_errorf};
use sqlx_context::DbContext;

/// 获取自己的文件,文件不存在或者不属于该用户时返回错误
pub async fn select_own(ctx: &mut DbContext, user_id: i64, id: i64) -> Result<DfsFile, BizError> {
    let file = match dfs_file_dao::select_one(&mut *ctx, id).await {
        Ok(it) => it,
        Err(sqlx::Error::RowNotFound) => return Err(biz_error!("文件夹或文件不存在")),
        Err(e) => return Err(e.into()),
    };
    if file.userId != user_id {
        return Err(biz_error!("非法操作"));
    }
    Ok(file)
}

/// 获取可以操作(重命名、移动、删除等)的文件
/// 根目录、附属文件、历史版本、已删除的文件都不允许直接操作
pub async fn select_operable(ctx: &mut DbContext, user_id: i64, id: i64) -> Result<DfsFile, BizError> {
    let file = select_own(ctx, user_id, id).await?;
    if file.parentId == 0 || file.isExtra == 1 || file.isHistory == 1 || file.deleteDate.is_some() {
        return Err(biz_error!("非法操作"));
    }
    Ok(file)
}

/// 通过路径获取文件或文件夹
/// - path 文件路径,空字符串或者/代表根目录
pub async fn select_by_path(ctx: &mut DbContext, user_id: i64, path: &str) -> Result<Option<DfsFile>, BizError> {
    let mut parent_id = 0;
    let mut file = None;
    for name in file_util::to_sub_names(path)? {
        let Some(it) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, user_id, parent_id, name).await? else {
            return Ok(None);
        };
        parent_id = it.id;
        file = Some(it);
    }
    Ok(file)
}

/// 通过路径获取文件夹ID
/// - is_create 文件夹不存在时是否创建
pub async fn get_folder_id(ctx: &mut DbContext, user_id: i64, folder: &str, is_create: bool) -> Result<Option<i64>, BizError> {
    if let Some(it) = select_by_path(ctx, user_id, folder).await? {
        if it.storageId > 0 {
            return Err(biz_errorf!("{}是一个文件,不是文件夹", folder));
        }
        return Ok(Some(it.id));
    }
    if !is_create {
        return Ok(None);
    }
    Ok(Some(mkdirs(ctx, user_id, folder).await?))
}

/// 创建文件夹,上级文件夹不存在时一起创建
/// - return 最后一级文件夹的ID
pub async fn mkdirs(ctx: &mut DbContext, user_id: i64, path: &str) -> Result<i64, BizError> {
    let mut parent_id = 0;
    let mut cur_path = String::new();
    for (index, name) in file_util::to_sub_names(path)?.into_iter().enumerate() {
        if index > 0 {
            cur_path.push('/');
            cur_path.push_str(name);
            file_util::check_name(name)?;
        }
        if let Some(it) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, user_id, parent_id, name).await? {
            if it.storageId > 0 {
                return Err(biz_errorf!("{}是一个文件,不允许创建文件夹", cur_path));
            }
            parent_id = it.id;
            continue;
        }
        if index == 0 {
            return Err(biz_error!("用户根目录不存在"));
        }
        parent_id = add_folder(ctx, user_id, parent_id, name).await?;
    }
    Ok(parent_id)
}

/// 添加一个文件夹
async fn add_folder(ctx: &mut DbContext, user_id: i64, parent_id: i64, name: &str) -> Result<i64, BizError> {
    let folder = DfsFile {
        id: id_util::id(),
        userId: user_id,
        parentId: parent_id,
        name: name.to_string(),
        date: time_util::current_millis(),
        ..Default::default()
    };
    dfs_file_dao::insert(&mut *ctx, &folder).await?;
    Ok(folder.id)
}

/// 添加一个文件
/// - is_over_write 是否覆盖,true时,将原来的文件设置为历史版本
pub async fn add_file(ctx: &mut DbContext, mut file: DfsFile, is_over_write: bool) -> Result<(), BizError> {
    if file.storageId == 0 {
        return Err(biz_error!("本地存储文件ID不能为空"));
    }
    file_util::check_name(&file.name)?;
    if let Some(exists) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, file.userId, file.parentId, &file.name).await? {
        if exists.storageId == 0 {
            return Err(biz_errorf!("已存在同名文件夹:{}", file.name));
        }
        if exists.storageId == file.storageId {
            //同一个文件,直接成功
            return Ok(());
        }
        if !is_over_write {
            return Err(biz_errorf!("文件[{}]已存在", file.name));
        }

        //将已经存在的文件标记为历史版本
        dfs_file_dao::set_history(&mut *ctx, exists.id).await?;
    }
    file.id = id_util::id();
    file.date = time_util::current_millis();
    file.ext = file_util::file_ext(&file.name);
    file.isExtra = 0;
    file.isHistory = 0;
    file.deleteDate = None;
    dfs_file_dao::insert(&mut *ctx, &file).await?;
    Ok(())
}

/// 通过文件ID获取文件全路径,根目录为空字符串
pub async fn get_path_by_id(ctx: &mut DbContext, id: i64) -> Result<String, BizError> {
    let mut path = String::new();
    let mut temp_id = id;
    loop {
        let file = dfs_file_dao::select_one(&mut *ctx, temp_id).await?;
        if file.parentId == 0 {
            break;
        }
        path = format!("/{}{}", file.name, path);
        temp_id = file.parentId;
    }
    Ok(path)
}

/// 判断目标文件夹是否是该文件夹自己或者它的子文件夹
pub async fn is_self_or_sub_folder(ctx: &mut DbContext, folder_id: i64, target_folder_id: i64) -> Result<bool, BizError> {
    let mut temp_id = target_folder_id;
    while temp_id != 0 {
        if temp_id == folder_id {
            return Ok(true);
        }
        temp_id = dfs_file_dao::select_one(&mut *ctx, temp_id).await?.parentId;
    }
    Ok(false)
}

/// 同一个文件夹下复制时,为新的文件或文件夹加上编号
/// 例: test.zip  ==>  test(1).zip
async fn make_name_no(ctx: &mut DbContext, file: &DfsFile) -> Result<String, BizError> {
    let (start_name, end_name) = match file.name.rfind('.') {
        Some(index) if file.storageId > 0 => (&file.name[..index], &file.name[index..]),
        _ => (file.name.as_str(), ""),
    };
    for i in 1..10000 {
        let name = format!("{}({}){}", start_name, i, end_name);
        if dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, file.userId, file.parentId, &name).await?.is_none() {
            return Ok(name);
        }
    }
    Err(biz_errorf!("文件[{}]的副本太多", file.name))
}

/// 复制文件或文件夹
/// - ids 要复制的文件ID
/// - target_folder 目标文件夹路径
/// - is_over_write 是否覆盖目标文件
pub async fn copy(ctx: &mut DbContext, user_id: i64, ids: &[i64], target_folder: &str, is_over_write: bool) -> Result<(), BizError> {
    let target_id = get_folder_id(ctx, user_id, target_folder, true).await?.unwrap();
    for id in ids {
        let source = select_operable(ctx, user_id, *id).await?;
        let name = if source.parentId == target_id {
            //同一个文件夹下复制时,自动加上编号
            make_name_no(ctx, &source).await?
        } else {
            source.name.clone()
        };
        if source.storageId == 0 && is_self_or_sub_folder(ctx, source.id, target_id).await? {
            return Err(biz_error!("不能复制文件夹到它自己或者它的子文件夹下"));
        }
        copy_one(ctx, source, user_id, target_id, name, is_over_write).await?;
    }
    Ok(())
}

/// 复制一个文件或文件夹(文件夹时递归复制所有子文件)
/// - target_user_id 复制到哪个用户(转存分享的文件时,与源文件的用户不同)
pub async fn copy_one(
    ctx: &mut DbContext,
    source: DfsFile,
    target_user_id: i64,
    target_parent_id: i64,
    name: String,
    is_over_write: bool,
) -> Result<(), BizError> {
    if source.storageId > 0 {
        let file = DfsFile {
            userId: target_user_id,
            parentId: target_parent_id,
            name,
            size: source.size,
            contentType: source.contentType,
            storageId: source.storageId,
            ..Default::default()
        };
        return add_file(ctx, file, is_over_write).await;
    }
    let folder_id = match dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, target_user_id, target_parent_id, &name).await? {
        Some(it) if it.storageId > 0 => return Err(biz_errorf!("文件或文件夹[{}]已存在", name)),
        Some(it) => it.id,
        None => add_folder(ctx, target_user_id, target_parent_id, &name).await?,
    };
    for it in dfs_file_dao::select_sub_list(&mut *ctx, source.userId, source.id).await? {
        let name = it.name.clone();
        Box::pin(copy_one(ctx, it, target_user_id, folder_id, name, is_over_write)).await?;
    }
    Ok(())
}

/// 移动文件或文件夹
/// - ids 要移动的文件ID
/// - target_folder 目标文件夹路径
/// - is_over_write 是否覆盖目标文件
pub async fn move_to(ctx: &mut DbContext, user_id: i64, ids: &[i64], target_folder: &str, is_over_write: bool) -> Result<(), BizError> {
    let target_id = get_folder_id(ctx, user_id, target_folder, true).await?.unwrap();
    for id in ids {
        let source = select_operable(ctx, user_id, *id).await?;
        if source.parentId == target_id {
            //同一个文件夹无需操作
            continue;
        }
        if source.storageId == 0 && is_self_or_sub_folder(ctx, source.id, target_id).await? {
            return Err(biz_error!("不能移动文件夹到它自己或者它的子文件夹下"));
        }
        move_one(ctx, source, target_id, is_over_write).await?;
    }
    Ok(())
}

/// 移动一个文件或文件夹
async fn move_one(ctx: &mut DbContext, source: DfsFile, target_parent_id: i64, is_over_write: bool) -> Result<(), BizError> {
    let exists = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, source.userId, target_parent_id, &source.name).await?;
    if source.storageId > 0 {
        if let Some(exists) = exists {
            if exists.storageId == 0 || !is_over_write {
                return Err(biz_errorf!("文件或文件夹[{}]已存在", source.name));
            }

            //将已经存在的文件标记为历史版本
            dfs_file_dao::set_history(&mut *ctx, exists.id).await?;
        }
        let param = MoveFileWithHistoryParam {
            user_id: source.userId,
            parent_id: source.parentId,
            name: source.name.clone(),
            target_parent_id,
            target_name: source.name,
        };
        dfs_file_dao::move_file_with_history(&mut *ctx, param).await?;
        return Ok(());
    }
    let Some(exists) = exists else {
        //目标文件夹不存在同名文件夹时,直接移动整个文件夹
        dfs_file_dao::set_parent_and_name(&mut *ctx, source.id, target_parent_id, &source.name).await?;
        return Ok(());
    };
    if exists.storageId > 0 {
        return Err(biz_errorf!("文件或文件夹[{}]已存在", source.name));
    }

    //目标位置已经存在同名文件夹时,合并两个文件夹
    for it in dfs_file_dao::select_sub_list(&mut *ctx, source.userId, source.id).await? {
        Box::pin(move_one(ctx, it, exists.id, is_over_write)).await?;
    }

    //剩下的是垃圾箱里的文件,也一起移过去,然后删除源文件夹
    for it in dfs_file_dao::select_all_child_list(&mut *ctx, source.id).await? {
        dfs_file_dao::set_parent_and_name(&mut *ctx, it.id, exists.id, &it.name).await?;
    }
    dfs_file_dao::delete(&mut *ctx, source.id).await?;
    Ok(())
}

/// 重命名
pub async fn rename(ctx: &mut DbContext, user_id: i64, id: i64, name: &str) -> Result<(), BizError> {
    file_util::check_name(name)?;
    let file = select_operable(ctx, user_id, id).await?;
    if let Some(exists) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, user_id, file.parentId, name).await? {
        if exists.id != file.id {
            //有可能仅仅是将文件名大小写转换
            return Err(biz_errorf!("文件或文件夹[{}]已存在", name));
        }
    }
    if file.storageId > 0 {
        let param = MoveFileWithHistoryParam {
            user_id,
            parent_id: file.parentId,
            name: file.name,
            target_parent_id: file.parentId,
            target_name: name.to_string(),
        };
        dfs_file_dao::move_file_with_history(&mut *ctx, param).await?;
    } else {
        dfs_file_dao::set_parent_and_name(&mut *ctx, file.id, file.parentId, name).await?;
    }
    Ok(())
}

/// 删除文件或文件夹到垃圾箱(文件的历史版本一起删除)
pub async fn set_delete(ctx: &mut DbContext, user_id: i64, id: i64) -> Result<(), BizError> {
    let file = select_operable(ctx, user_id, id).await?;
    dfs_file_dao::set_delete(&mut *ctx, user_id, file.parentId, &file.name, time_util::current_millis()).await?;
    Ok(())
}

/// 从垃圾箱还原文件
pub async fn trash_recover(ctx: &mut DbContext, user_id: i64, ids: &[i64]) -> Result<(), BizError> {
    for id in ids {
        let file = select_own(ctx, user_id, *id).await?;
        let Some(delete_date) = file.deleteDate else {
            return Err(biz_error!("非法操作"));
        };
        if dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, user_id, file.parentId, &file.name).await?.is_some() {
            let path = get_path_by_id(ctx, file.id).await?;
            return Err(biz_errorf!("文件[{}]已存在", path));
        }
        dfs_file_dao::set_not_delete(&mut *ctx, user_id, file.parentId, &file.name, delete_date).await?;
    }
    Ok(())
}
