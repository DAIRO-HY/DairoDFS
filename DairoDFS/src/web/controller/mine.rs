use crate::application;
use crate::dao::user_dao;
use crate::db;
use crate::extension::number::{NumberExtension, ShortString};
use crate::util::time_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use base64::Engine;
use hyperoute_core::Ext;
use hyperoute_macro::{GET, PUT};
use rand::distr::{Alphanumeric, SampleString};
use rand::seq::SliceRandom;

/// 获取当前登录用户的信息
#[GET("/api/mine/info")]
pub async fn info(login: Ext<LoginUser>) -> Result<model::MineInfo, BizError> {
    let user = user_dao::select_one(&mut lib_db::get_context(), login.id).await?;
    Ok(model::MineInfo {
        id: user.id,
        name: user.name.unwrap_or_default(),
        email: user.email.unwrap_or_default(),
        date: user.date.date_format(),
        url_path: user.urlPath.unwrap_or_default(),
        api_token: user.apiToken.unwrap_or_default(),
        encryption_key: user.encryptionKey.unwrap_or_default(),
        is_admin: login.is_admin,
    })
}

/// 生成或关闭WEB访问路径前缀
/// - flag 1:生成 0:关闭
#[PUT("/api/mine/url_path")]
pub async fn url_path(login: Ext<LoginUser>, flag: i64) -> Result<(), BizError> {
    let url_path = if flag == 0 {
        None
    } else {
        Some((time_util::current_millis() - application::BASE_TIME).to_short_string())
    };
    let mut ctx = db::begin().await?;
    user_dao::set_url_path(&mut ctx, login.id, url_path).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 生成或关闭API票据
/// - flag 1:生成 0:关闭
#[PUT("/api/mine/api_token")]
pub async fn api_token(login: Ext<LoginUser>, flag: i64) -> Result<(), BizError> {
    let api_token = if flag == 0 {
        None
    } else {
        Some(Alphanumeric.sample_string(&mut rand::rng(), 32))
    };
    let mut ctx = db::begin().await?;
    user_dao::set_api_token(&mut ctx, login.id, api_token).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 生成或关闭端对端加密密钥
/// 密钥为0~127打乱顺序后的128个字节,再做base64编码
/// - flag 1:生成 0:关闭
#[PUT("/api/mine/encryption_key")]
pub async fn encryption_key(login: Ext<LoginUser>, flag: i64) -> Result<(), BizError> {
    let encryption_key = if flag == 0 {
        None
    } else {
        let mut data: Vec<u8> = (0..128).collect();
        data.shuffle(&mut rand::rng());
        Some(base64::engine::general_purpose::STANDARD.encode(data))
    };
    let mut ctx = db::begin().await?;
    user_dao::set_encryption_key(&mut ctx, login.id, encryption_key).await?;
    db::commit(ctx).await?;
    Ok(())
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MineInfo {

        // 用户ID
        pub id: i64,

        // 用户名
        pub name: String,

        // 邮箱
        pub email: String,

        // 创建日期
        pub date: String,

        // 用户文件访问路径前缀
        pub url_path: String,

        // API操作TOKEN
        pub api_token: String,

        // 端对端加密密钥
        pub encryption_key: String,

        // 是否管理员
        pub is_admin: bool,
    }
}
