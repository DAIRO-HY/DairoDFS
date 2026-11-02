use crate::application;
use crate::dao::user_dao;
use crate::db;
use crate::extension::number::{NumberExtension, ShortString};
use crate::util::time_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use base64::Engine;
use hyperoute_core::{Ext, MultipartFile, Stream};
use hyper::HeaderMap;
use std::path::Path;
use crate::dao::storage_file_dao;
use crate::extension::string::StringExtension;
use crate::service::storage_service;
use crate::util::id_util;
use crate::util::media;
use crate::util::media::image_util;
use crate::web::form;
use crate::biz_error;
use hyperoute_macro::{DELETE, GET, POST, PUT};
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
        is_sub: login.is_sub,
        nickname: user.nickname.unwrap_or_default(),
        avatar: user.avatar.unwrap_or_default(),
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

/// 修改昵称
#[PUT("/api/mine/nickname")]
pub async fn nickname(login: Ext<LoginUser>, nickname: Option<String>) -> Result<(), BizError> {
    let nickname = form::none_if_empty(nickname);
    if nickname.as_ref().is_some_and(|it| it.chars().count() > 32) {
        return Err(BizError::field("nickname", "昵称长度不能超过32位"));
    }
    let mut ctx = db::begin().await?;
    user_dao::set_nickname(&mut ctx, login.id, nickname).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 头像图片最大的文件大小
const AVATAR_MAX_SIZE: u64 = 20 * 1024 * 1024;

/// 头像图片的尺寸
const AVATAR_SIZE: u32 = 256;

/// 上传头像,图片会裁剪成正方形
#[POST("/api/mine/avatar")]
pub async fn upload_avatar(login: Ext<LoginUser>, file: MultipartFile<&Path>) -> Result<String, BizError> {
    if tokio::fs::metadata(file.body).await?.len() > AVATAR_MAX_SIZE {
        return Err(biz_error!("图片不能超过20MB"));
    }
    let data = tokio::fs::read(file.body).await?;
    let data = media::blocking(move || image_util::crop(&data, AVATAR_SIZE, AVATAR_SIZE, 90))
        .await
        .map_err(|_| biz_error!("不支持的图片格式"))?;
    let md5 = format!("{:x}", md5::compute(&data));

    //和普通文件一样存放到存储目录,分布式同步时分机也能获取到
    let temp_path = application::TEMP_PATH.join(format!("avatar-{}", id_util::id()));
    tokio::fs::write(&temp_path, &data).await?;
    storage_service::save(&temp_path, &md5).await?;
    let mut ctx = db::begin().await?;
    user_dao::set_avatar(&mut ctx, login.id, Some(md5.clone())).await?;
    db::commit(ctx).await?;
    Ok(md5)
}

/// 删除头像
#[DELETE("/api/mine/avatar")]
pub async fn delete_avatar(login: Ext<LoginUser>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    user_dao::set_avatar(&mut ctx, login.id, None).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 获取头像
/// 可以获取自己的头像;主账户可以获取子账户的头像;子账户可以获取所属主账户的头像;管理员可以获取所有人的头像
#[GET("/api/avatar/{id}")]
pub async fn avatar(login: Ext<LoginUser>, id: i64, headers: &HeaderMap) -> Result<Stream, BizError> {
    let mut ctx = lib_db::get_context();
    let Ok(user) = user_dao::select_one(&mut ctx, id).await else {
        return Err(BizError::not_found());
    };
    let allow = login.is_admin || user.id == login.id || user.id == login.owner_id || (!login.is_sub && user.parentId == login.id);
    if !allow {
        return Err(BizError::not_found());
    }
    let Some(md5) = user.avatar else {
        return Err(BizError::not_found());
    };
    let Some(storage) = storage_file_dao::select_by_md5(&mut ctx, &md5).await? else {
        return Err(BizError::not_found());
    };
    let Ok(stream) = Stream::file_with_etag(&storage.path, headers, &md5.md5()).await else {
        return Err(BizError::not_found());
    };
    Ok(stream.content_type("image/jpeg").set_header("cache-control", "private, no-cache"))
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

        // 是否子账户
        pub is_sub: bool,

        // 昵称
        pub nickname: String,

        // 头像的版本(内容变化时改变),为空表示没有头像,通过 /api/avatar/{用户ID} 获取
        pub avatar: String,
    }
}
