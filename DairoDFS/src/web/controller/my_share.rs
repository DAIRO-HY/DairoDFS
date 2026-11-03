use crate::dao::{share_dao, user_dao};
use std::collections::HashMap;
use crate::extension::number::NumberExtension;
use crate::service::share_service;
use crate::service::operate_log_service::{self, op};
use crate::service::sub_account_service::{perm, Scope};
use crate::service::dfs_file_service;
use crate::util::time_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{biz_error, db};
use hyperoute_core::Ext;
use hyperoute_macro::{DELETE, GET, POST};

/// 创建分享
/// - ids 要分享的文件ID,必须在同一个文件夹下
/// - end_date_time 分享截止时间戳,0代表永久有效
/// - pwd 提取码,不填代表不需要提取码
#[POST("/api/files/share")]
pub async fn share(login: Ext<LoginUser>, ids: Vec<i64>, end_date_time: i64, pwd: Option<String>) -> Result<i64, BizError> {
    let mut ctx = db::begin().await?;
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut files = Vec::new();
    for id in &ids {
        let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, *id).await?;
        scope.check(&mut ctx, &file, perm::SHARE).await?;
        let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
        files.push((file, path));
    }
    let creator_id = login.is_sub.then_some(login.id);
    let id = share_service::add(&mut ctx, login.owner_id, &ids, end_date_time, pwd.unwrap_or_default().trim(), creator_id).await?;
    db::commit(ctx).await?;
    for (file, path) in files {
        operate_log_service::add(&login, op::SHARE, Some(file.id), Some(path), Some(serde_json::json!({"shareId": id})));
    }
    Ok(id)
}

/// 获取我的分享列表
#[GET("/api/my_share/list")]
pub async fn list(login: Ext<LoginUser>) -> Result<Vec<model::MyShare>, BizError> {
    let now = time_util::current_millis();
    let mut ctx = lib_db::get_context();

    //创建分享的子账户名称(子账户已经删除时用户名为空,显示昵称)
    let mut creator_names = HashMap::new();
    if !login.is_sub {
        for it in user_dao::select_sub_list_all(&mut ctx, login.id).await? {
            creator_names.insert(it.id, it.name.or(it.nickname).unwrap_or_else(|| "已删除的子账户".to_string()));
        }
    }
    let list = share_dao::select_by_user_id(&mut ctx, login.owner_id)
        .await?
        .into_iter()
        .filter(|it| is_own(&login, it))
        .map(|it| {
            let end_date = match it.endDate.unwrap_or(0) {
                0 => "永久有效".to_string(),
                end_date => {
                    let end_time = end_date - now;
                    if end_time < 0 {
                        "已过期".to_string()
                    } else if end_time > 24 * 60 * 60 * 1000 {
                        format!("{}天后过期", end_time / (24 * 60 * 60 * 1000))
                    } else if end_time > 60 * 60 * 1000 {
                        format!("{}小时后过期", end_time / (60 * 60 * 1000))
                    } else if end_time > 60 * 1000 {
                        format!("{}分钟后过期", end_time / (60 * 1000))
                    } else {
                        "即将过期".to_string()
                    }
                }
            };
            model::MyShare {
                id: it.id,
                title: it.title.unwrap_or_default(),
                file_count: it.fileCount,
                folder_flag: it.folderFlag == Some(1),
                end_date,
                date: it.date.date_format(),
                thumb: it.thumb.filter(|it| *it > 0),
                creator_name: it.creatorId.and_then(|id| creator_names.get(&id).cloned()),
            }
        })
        .collect();
    Ok(list)
}

/// 获取分享详细
#[GET("/api/my_share/detail")]
pub async fn detail(login: Ext<LoginUser>, id: i64) -> Result<model::MyShareDetail, BizError> {
    let Ok(share) = share_dao::select_one(&mut lib_db::get_context(), id).await else {
        return Err(biz_error!("分享链接不存在"));
    };
    if !is_own(&login, &share) {
        return Err(biz_error!("非法操作"));
    }
    Ok(model::MyShareDetail {
        id: share.id,
        url: share_service::share_url(share.id),
        pwd: share.pwd.filter(|it| !it.is_empty()).unwrap_or_else(|| "无".to_string()),
        folder: if share.folder.is_empty() { "/".to_string() } else { share.folder },
        names: share.names,
        end_date: match share.endDate.unwrap_or(0) {
            0 => "永久有效".to_string(),
            it => it.date_format(),
        },
        date: share.date.date_format(),
    })
}

/// 取消分享
/// - ids 分享ID
#[DELETE("/api/my_share/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    if ids.is_empty() {
        //ids为空时会删除该用户的所有分享
        return Err(biz_error!("请选择要取消的分享"));
    }
    let mut ctx = db::begin().await?;
    let mut shares = Vec::new();
    for id in &ids {
        match share_dao::select_one(&mut ctx, *id).await {
            Ok(it) if is_own(&login, &it) => shares.push(it),
            _ => return Err(biz_error!("非法操作")),
        }
    }
    share_dao::delete_by_user_id_and_ids(&mut ctx, login.owner_id, ids).await?;
    db::commit(ctx).await?;
    for it in shares {
        let path = format!("{}/{}", it.folder, it.names.replace('|', ","));
        operate_log_service::add(&login, op::SHARE_DELETE, None, Some(path), Some(serde_json::json!({"shareId": it.id})));
    }
    Ok(())
}

/// 是否可以管理的分享:主账户可以管理自己和子账户创建的分享,子账户只能管理自己创建的分享
fn is_own(login: &LoginUser, share: &share_dao::Share) -> bool {
    share.userId == login.owner_id && (!login.is_sub || share.creatorId == Some(login.id))
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MyShare {

        // 分享ID
        pub id: i64,

        // 分享的标题(文件名)
        pub title: String,

        // 文件数量
        pub file_count: i64,

        // 是否分享的仅仅是一个文件夹
        pub folder_flag: bool,

        // 结束时间
        pub end_date: String,

        // 创建日期
        pub date: String,

        // 缩略图的文件ID
        pub thumb: Option<i64>,

        // 创建分享的子账户名称,主账户自己创建的为空
        pub creator_name: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MyShareDetail {

        // 分享ID
        pub id: i64,

        // 分享链接
        pub url: String,

        // 提取码
        pub pwd: String,

        // 分享的文件所在的文件夹
        pub folder: String,

        // 分享的文件夹或文件名,用|分割
        pub names: String,

        // 结束日期
        pub end_date: String,

        // 创建日期
        pub date: String,
    }
}
