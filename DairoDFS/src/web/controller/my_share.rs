use crate::dao::share_dao;
use crate::extension::number::NumberExtension;
use crate::service::share_service;
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
    let id = share_service::add(&mut ctx, login.id, &ids, end_date_time, pwd.unwrap_or_default().trim()).await?;
    db::commit(ctx).await?;
    Ok(id)
}

/// 获取我的分享列表
#[GET("/api/my_share/list")]
pub async fn list(login: Ext<LoginUser>) -> Result<Vec<model::MyShare>, BizError> {
    let now = time_util::current_millis();
    let list = share_dao::select_by_user_id(&mut lib_db::get_context(), login.id)
        .await?
        .into_iter()
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
    if share.userId != login.id {
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
    share_dao::delete_by_user_id_and_ids(&mut ctx, login.id, ids).await?;
    db::commit(ctx).await?;
    Ok(())
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
