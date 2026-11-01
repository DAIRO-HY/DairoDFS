use crate::dao::{user_dao, user_token_dao};
use crate::util::request_util;
use crate::web::biz_error::BizError;
use hyper::HeaderMap;
use hyperoute_core::{Next, Request, Response};
use hyperoute_macro::FILTER;

/// 当前登录的用户,登录验证通过后放进请求扩展里,handler通过 Ext<LoginUser> 获取
#[derive(Debug, Clone)]
pub struct LoginUser {
    // 用户ID
    pub id: i64,

    // 是否管理员
    pub is_admin: bool,
}

/// 验证是否已经登录
#[FILTER(
    order = 10,
    include = ["/api/**"],
    exclude = ["/api/login/do_login", "/api/login/forget", "/api/common/global", "/api/install/**", "/api/share/**"]
)]
pub async fn auth_login(mut req: Request, next: Next) -> Result<Response, BizError> {
    let Some(login_user) = find_login_user(req.headers()).await? else {
        return Err(BizError::unauthorized());
    };
    req.extensions_mut().insert(login_user);
    Ok(next.run(req).await)
}

/// 通过请求头里的登录票据获取当前登录的用户,没有登录或者用户已经被禁用时返回None
pub async fn find_login_user(headers: &HeaderMap) -> Result<Option<LoginUser>, BizError> {
    let Some(token) = request_util::get_token(headers) else {
        return Ok(None);
    };
    if token.is_empty() {
        //token为空,说明没有登录
        return Ok(None);
    }
    let mut ctx = lib_db::get_context();

    //先从登录记录里查找,找不到时再当作API票据查找
    let user_id = match user_token_dao::select_user_id_by_token(&mut ctx, token).await? {
        Some(it) => it,
        None => match user_dao::select_id_by_api_token(&mut ctx, token).await? {
            Some(it) => it,
            None => return Ok(None),
        },
    };
    let Ok(user) = user_dao::select_one(&mut ctx, user_id).await else {
        return Ok(None);
    };
    if user.state != 1 {
        //用户已经被禁用
        return Ok(None);
    }
    let admin_id = user_dao::select_admin_id(&mut ctx).await?;
    Ok(Some(LoginUser {
        id: user_id,
        is_admin: admin_id == Some(user_id),
    }))
}

/// 验证是否管理员
#[FILTER(order = 20, include = ["/api/user/**", "/api/profile/**", "/api/sync/**", "/api/advanced/**"])]
pub async fn auth_admin(req: Request, next: Next) -> Result<Response, BizError> {
    let is_admin = req
        .extensions()
        .get::<LoginUser>()
        .is_some_and(|it| it.is_admin);
    if !is_admin {
        return Err(BizError::forbidden());
    }
    Ok(next.run(req).await)
}
