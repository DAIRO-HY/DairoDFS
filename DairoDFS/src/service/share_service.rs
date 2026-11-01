use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::share_dao::Share;
use crate::dao::{dfs_file_dao, share_dao};
use crate::extension::string::StringExtension;
use crate::service::dfs_file_service;
use crate::util::{aes_util, file_util, id_util, request_util, time_util};
use crate::web::biz_error::BizError;
use crate::{biz_error, biz_errorf};
use hyper::HeaderMap;
use sqlx_context::DbContext;
use std::collections::HashSet;

/// 提取码验证通过后保存在Cookie里的名称
pub const SHARE_PWD_COOKIE: &str = "share_pwd";

/// 分享链接(与老版本一致)
pub fn share_url(id: i64) -> String {
    format!("/share/{}/init", aes_util::encrypt(&id.to_string()))
}

/// 创建分享
/// - ids 要分享的文件ID,必须在同一个文件夹下
/// - end_date_time 分享截止时间戳,0代表永久有效
/// - pwd 提取码,空字符串代表不需要提取码
pub async fn add(ctx: &mut DbContext, user_id: i64, ids: &[i64], end_date_time: i64, pwd: &str) -> Result<i64, BizError> {
    if ids.is_empty() {
        return Err(biz_error!("请选择要分享的文件"));
    }
    if end_date_time != 0 && end_date_time < time_util::current_millis() {
        return Err(BizError::field("endDateTime", "结束日期必须在现在的时间之后"));
    }
    if pwd.chars().count() > 32 {
        return Err(BizError::field("pwd", "提取码不能超过32位"));
    }
    let mut files = Vec::new();
    for id in ids {
        files.push(dfs_file_service::select_operable(ctx, user_id, *id).await?);
    }
    let parent_id = files[0].parentId;
    if files.iter().any(|it| it.parentId != parent_id) {
        return Err(biz_error!("分享的文件必须在同一个文件夹下"));
    }

    //缩略图使用第一个有缩略图的文件
    let mut thumb = None;
    for it in &files {
        if it.storageId > 0 && dfs_file_dao::select_extra(&mut *ctx, it.id, "thumb").await?.is_some() {
            thumb = Some(it.id);
            break;
        }
    }
    let names: Vec<&str> = files.iter().map(|it| it.name.as_str()).collect();
    let title = if names.len() == 1 {
        names[0].to_string()
    } else {
        format!("{}等多个文件", names[0])
    };
    let share = Share {
        id: id_util::id(),
        title: Some(title),
        userId: user_id,
        pwd: Some(pwd.to_string()).filter(|it| !it.is_empty()),
        folder: dfs_file_service::get_path_by_id(ctx, parent_id).await?,
        names: names.join("|"),
        thumb,
        folderFlag: Some((files.len() == 1 && files[0].storageId == 0) as i64),
        fileCount: files.len() as i64,
        endDate: Some(end_date_time),
        date: time_util::current_millis(),
    };
    share_dao::insert(&mut *ctx, &share).await?;
    Ok(share.id)
}

/// 访问分享时的错误
pub enum ShareError {
    // 业务错误(分享不存在、已过期等)
    Biz(BizError),

    // 需要提取码
    NeedPwd(Share),
}

impl From<BizError> for ShareError {
    fn from(err: BizError) -> Self {
        ShareError::Biz(err)
    }
}

impl From<sqlx::Error> for ShareError {
    fn from(err: sqlx::Error) -> Self {
        ShareError::Biz(err.into())
    }
}

/// 获取分享信息,验证是否过期,以及提取码是否正确
pub async fn get(eid: &str, headers: &HeaderMap) -> Result<Share, ShareError> {
    let share = get_without_pwd(eid).await?;
    let Some(pwd) = share.pwd.as_deref().filter(|it| !it.is_empty()) else {
        return Ok(share);
    };
    if request_util::get_cookie(headers, SHARE_PWD_COOKIE) != Some(encode_pwd(share.id, pwd).as_str()) {
        return Err(ShareError::NeedPwd(share));
    }
    Ok(share)
}

/// 获取分享信息,只验证是否过期
pub async fn get_without_pwd(eid: &str) -> Result<Share, BizError> {
    let not_found = || biz_error!("分享链接不存在");
    let id: i64 = aes_util::decrypt(eid).and_then(|it| it.parse().ok()).ok_or_else(not_found)?;
    let share = match share_dao::select_one(&mut lib_db::get_context(), id).await {
        Ok(it) => it,
        Err(sqlx::Error::RowNotFound) => return Err(not_found()),
        Err(e) => return Err(e.into()),
    };
    if share.endDate.is_some_and(|it| it != 0 && it < time_util::current_millis()) {
        return Err(biz_error!("分享已过期"));
    }
    Ok(share)
}

/// 提取码验证通过后保存在Cookie里的值(密钥参与计算,外部无法伪造)
pub fn encode_pwd(id: i64, pwd: &str) -> String {
    format!("{}{}{}", String::from_utf8_lossy(aes_util::key()), id, pwd).md5()
}

/// 获取分享里的文件
/// - path 相对于分享文件夹的路径,第一级必须是分享的文件或文件夹
pub async fn select_file(share: &Share, path: &str) -> Result<DfsFile, BizError> {
    let names = file_util::to_sub_names(path)?;
    let not_found = || biz_error!("文件夹或文件不存在");
    if names.len() < 2 || !share.names.split('|').any(|it| it == names[1]) {
        //不在分享范围内的文件
        return Err(not_found());
    }
    dfs_file_service::select_by_path(&mut lib_db::get_context(), share.userId, &format!("{}{}", share.folder, path))
        .await?
        .ok_or_else(not_found)
}

/// 获取分享的文件列表
/// - folder 相对于分享文件夹的路径,空字符串代表分享的根目录
pub async fn list(share: &Share, folder: &str) -> Result<Vec<DfsFile>, BizError> {
    let mut ctx = lib_db::get_context();
    if !folder.is_empty() {
        let file = select_file(share, folder).await?;
        if file.storageId > 0 {
            return Err(biz_errorf!("{}不是一个文件夹", folder));
        }
        return Ok(dfs_file_dao::select_sub_list(&mut ctx, share.userId, file.id).await?);
    }

    //分享的根目录,只显示分享的文件
    let Some(share_folder) = dfs_file_service::select_by_path(&mut ctx, share.userId, &share.folder).await? else {
        return Err(biz_error!("文件夹不存在"));
    };
    let names: HashSet<&str> = share.names.split('|').collect();
    Ok(dfs_file_dao::select_sub_list(&mut ctx, share.userId, share_folder.id)
        .await?
        .into_iter()
        .filter(|it| names.contains(it.name.as_str()))
        .collect())
}

/// 转存到自己的文件夹
/// - folder 分享里的文件夹(相对路径)
/// - names 选中的文件名
/// - target 转存到的目标文件夹
pub async fn save_to(ctx: &mut DbContext, share: &Share, user_id: i64, folder: &str, names: &[String], target: &str) -> Result<(), BizError> {
    if names.is_empty() {
        return Err(biz_error!("请选择要转存的文件"));
    }
    let target_id = dfs_file_service::get_folder_id(ctx, user_id, target, true).await?.unwrap();
    for name in names {
        let source = select_file(share, &format!("{}/{}", folder, name)).await?;
        if source.storageId == 0 && dfs_file_service::is_self_or_sub_folder(ctx, source.id, target_id).await? {
            //分享者转存自己分享的文件夹到它的子文件夹下
            return Err(biz_error!("不能转存文件夹到它自己或者它的子文件夹下"));
        }
        dfs_file_service::copy_one(ctx, source, user_id, target_id, name.clone(), false).await?;
    }
    Ok(())
}
