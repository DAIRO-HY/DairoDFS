use crate::application;
use crate::dao::user_token_dao::UserToken;
use crate::dao::{user_dao, user_token_dao};
use crate::extension::string::StringExtension;
use crate::util::{id_util, request_util, time_util};
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, db};
use hyper::HeaderMap;
use hyperoute_core::{Raw, RemoteAddr};
use hyperoute_macro::{DELETE, GET, POST};
use rand::distr::{Alphanumeric, SampleString};
use validator::Validate;

/// 登录API
#[POST("/api/login/do_login")]
pub async fn do_login(form: form::Login, headers: &HeaderMap, addr: RemoteAddr) -> Result<Raw, BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    let mut ctx = db::begin().await?;
    let Some(user) = user_dao::select_by_name(&mut ctx, form.name).await? else {
        return Err(biz_error!("用户名或密码错误"));
    };
    if user.pwd.as_deref() != Some(form.pwd.md5().as_str()) {
        return Err(biz_error!("用户名或密码错误"));
    }
    if user.state != 1 {
        return Err(biz_error!("该用户已被禁用"));
    }

    //同一台设备只保留一条登录记录
    user_token_dao::delete_by_user_id_and_device_id(&mut ctx, user.id, form.device_id.clone()).await?;

    // 生成一个随机的32位token，作为登录状态的标识
    let token = Alphanumeric.sample_string(&mut rand::rng(), 32);
    let user_token = UserToken {
        id: id_util::id(),
        token: token.clone(),
        userId: user.id,
        clientFlag: form.client_flag,
        deviceId: form.device_id,
        ip: request_util::get_ip(headers, &addr),
        date: time_util::current_millis(),
        client_version: form.client_version,
    };
    user_token_dao::insert(&mut ctx, &user_token).await?;

    //超过允许登录的客户端数量时,挤掉最早的登录记录
    let token_list = user_token_dao::select_by_user_id(&mut ctx, user.id).await?;
    if token_list.len() > application::USER_TOKEN_LIMIT {
        for it in &token_list[..token_list.len() - application::USER_TOKEN_LIMIT] {
            user_token_dao::delete_by_token(&mut ctx, it.token.clone()).await?;
        }
    }
    let admin_id = user_dao::select_admin_id(&mut ctx).await?;
    db::commit(ctx).await?;

    let data = model::LoginResult {
        token: token.clone(),
        name: user.name.unwrap_or_default(),
        is_admin: admin_id == Some(user.id),
    };

    // 写入 Cookie，全站生效，HttpOnly防止 JS 读取（安全）;APP从返回的数据里获取token
    Ok(Raw::new(serde_json::to_string(&data).unwrap())
        .header("content-type", "application/json")
        .header("set-cookie", request_util::token_cookie(&token)))
}

/// 退出登录
#[DELETE("/api/login/logout")]
pub async fn logout(headers: &HeaderMap) -> Result<Raw, BizError> {
    if let Some(token) = request_util::get_token(headers) {
        let mut ctx = db::begin().await?;
        user_token_dao::delete_by_token(&mut ctx, token.to_string()).await?;
        db::commit(ctx).await?;
    }
    Ok(Raw::new("").header("set-cookie", request_util::clear_token_cookie()))
}

/// 忘记密码
#[GET("/api/login/forget")]
pub async fn forget() -> String {
    "请联系管理员在用户管理中重置密码".to_string()
}

pub mod model {
    use serde::Serialize;

    /// 登录结果
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LoginResult {

        // 登录票据(APP使用,网页端使用Cookie)
        pub token: String,

        // 用户名
        pub name: String,

        // 是否管理员
        pub is_admin: bool,
    }
}
