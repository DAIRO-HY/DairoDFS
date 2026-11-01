use crate::dao::dfs_file_dao;
use crate::service::share_service::ShareError;
use crate::service::{file_handle_service, share_service};
use crate::web::biz_error::BizError;
use crate::web::controller::files;
use crate::web::filter::auth;
use crate::{biz_error, db};
use hyper::{HeaderMap, StatusCode};
use hyperoute_core::{Raw, Stream};
use hyperoute_macro::{GET, POST};
use std::collections::HashSet;

/// 分享链接(与老版本的链接格式一致),跳转到分享页面
#[GET("/share/{eid}/init")]
pub async fn init(eid: String) -> Raw {
    Raw::new("")
        .status(StatusCode::FOUND)
        .header("location", format!("/share.html?eid={}", eid))
}

/// 获取分享的基本信息
/// state: ok:可以访问 need_pwd:需要提取码 error:分享不存在或已过期等,原因在msg里
#[GET("/api/share/{eid}/info")]
pub async fn info(eid: String, headers: &HeaderMap) -> Result<model::ShareInfo, BizError> {
    let is_login = auth::find_login_user(headers).await?.is_some();
    let (share, state) = match share_service::get(&eid, headers).await {
        Ok(it) => (it, "ok"),
        Err(ShareError::NeedPwd(it)) => (it, "need_pwd"),
        Err(ShareError::Biz(e)) => {
            return Ok(model::ShareInfo {
                state: "error".to_string(),
                msg: e.to_string(),
                title: String::new(),
                is_login,
            });
        }
    };
    Ok(model::ShareInfo {
        state: state.to_string(),
        msg: String::new(),
        title: share.title.unwrap_or_default(),
        is_login,
    })
}

/// 验证提取码,验证成功后把加密的提取码保存到Cookie
#[POST("/api/share/{eid}/valid_pwd")]
pub async fn valid_pwd(eid: String, pwd: String) -> Result<Raw, BizError> {
    let share = share_service::get_without_pwd(&eid).await?;
    let share_pwd = share.pwd.unwrap_or_default();
    if share_pwd.is_empty() {
        return Ok(Raw::new(""));
    }
    if share_pwd != pwd.trim() {
        return Err(BizError::field("pwd", "提取码错误"));
    }

    //每个分享单独保存,同时打开多个分享时不会互相覆盖
    let cookie = format!(
        "{}={}; Path=/api/share/{}; HttpOnly; SameSite=Lax; Max-Age=36000",
        share_service::SHARE_PWD_COOKIE,
        share_service::encode_pwd(share.id, &share_pwd),
        eid
    );
    Ok(Raw::new("").header("set-cookie", cookie))
}

/// 获取分享的文件列表
/// - folder 相对于分享文件夹的路径,空字符串代表分享的根目录
#[GET("/api/share/{eid}/list")]
pub async fn list(eid: String, folder: String, headers: &HeaderMap) -> Result<Vec<model::ShareFile>, BizError> {
    let share = get_share(&eid, headers).await?;
    let list = share_service::list(&share, &folder).await?;
    let mut ctx = lib_db::get_context();
    let mut thumb_ids = HashSet::new();
    for it in &list {
        if it.storageId > 0 && dfs_file_dao::select_extra(&mut ctx, it.id, "thumb").await?.is_some() {
            thumb_ids.insert(it.id);
        }
    }
    Ok(list
        .into_iter()
        .map(|it| model::ShareFile {
            has_thumb: thumb_ids.contains(&it.id),
            name: it.name,
            size: it.size,
            date: it.date,
            file_flag: it.storageId > 0,
        })
        .collect())
}

/// 下载分享的文件
/// - path 相对于分享文件夹的路径
/// - download 是否以附件的方式下载
#[GET("/api/share/{eid}/download/{*path}")]
pub async fn download(eid: String, path: String, download: Option<bool>, headers: &HeaderMap) -> Result<Stream, BizError> {
    let share = get_share(&eid, headers).await?;
    let Ok(file) = share_service::select_file(&share, &format!("/{}", path)).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 {
        return Err(BizError::not_found());
    }
    let stream = files::file_stream(&file, headers).await?;
    if download == Some(true) {
        return Ok(stream.attachment(&file.name));
    }
    Ok(stream)
}

/// 获取分享的文件的缩略图
#[GET("/api/share/{eid}/thumb/{*path}")]
pub async fn thumb(eid: String, path: String, headers: &HeaderMap) -> Result<Stream, BizError> {
    let share = get_share(&eid, headers).await?;
    let Ok(file) = share_service::select_file(&share, &format!("/{}", path)).await else {
        return Err(BizError::not_found());
    };
    let Some(thumb) = dfs_file_dao::select_extra(&mut lib_db::get_context(), file.id, "thumb").await? else {
        return Err(BizError::not_found());
    };
    files::file_stream(&thumb, headers).await
}

/// 转存到自己的文件夹(需要登录)
/// - folder 分享里的文件夹(相对路径)
/// - names 选中的文件名
/// - target 转存到的目标文件夹
#[POST("/api/share/{eid}/save_to")]
pub async fn save_to(eid: String, folder: String, names: Vec<String>, target: String, headers: &HeaderMap) -> Result<(), BizError> {
    let Some(login) = auth::find_login_user(headers).await? else {
        return Err(BizError::unauthorized());
    };
    let share = get_share(&eid, headers).await?;
    let mut ctx = db::begin().await?;
    share_service::save_to(&mut ctx, &share, login.id, &folder, &names, &target).await?;
    db::commit(ctx).await?;

    //转存的文件需要生成附属文件(同一个存储文件已经生成过的会直接复用)
    file_handle_service::notify();
    Ok(())
}

/// 获取分享信息,需要提取码时返回错误
async fn get_share(eid: &str, headers: &HeaderMap) -> Result<crate::dao::share_dao::Share, BizError> {
    match share_service::get(eid, headers).await {
        Ok(it) => Ok(it),
        Err(ShareError::NeedPwd(_)) => Err(biz_error!("需要提取码")),
        Err(ShareError::Biz(e)) => Err(e),
    }
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ShareInfo {

        // ok:可以访问 need_pwd:需要提取码 error:分享不存在或已过期等
        pub state: String,

        // state为error时的原因
        pub msg: String,

        // 分享标题
        pub title: String,

        // 访问者是否已经登录(登录后才能转存)
        pub is_login: bool,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ShareFile {

        // 文件名
        pub name: String,

        // 文件大小
        pub size: i64,

        // 创建日期(时间戳毫秒)
        pub date: i64,

        // 是否文件
        pub file_flag: bool,

        // 是否有缩略图
        pub has_thumb: bool,
    }
}
