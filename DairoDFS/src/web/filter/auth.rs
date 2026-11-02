use crate::dao::{user_dao, user_token_dao};
use crate::util::request_util;
use crate::web::biz_error::BizError;
use hyper::HeaderMap;
use hyperoute_core::{Next, RemoteAddr, Request, Response};
use hyperoute_macro::FILTER;

/// 当前登录的用户,登录验证通过后放进请求扩展里,handler通过 Ext<LoginUser> 获取
#[derive(Debug, Clone)]
pub struct LoginUser {
    // 登录的用户ID(子账户登录时是子账户的ID)
    pub id: i64,

    // 登录的用户名
    pub name: String,

    // 文件所属的主账户ID,主账户登录时和id相同
    pub owner_id: i64,

    // 是否管理员
    pub is_admin: bool,

    // 是否子账户
    pub is_sub: bool,

    // 客户端IP地址
    pub ip: String,

    // 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX -1:API票据
    pub client_flag: i64,
}

/// 验证是否已经登录
#[FILTER(
    order = 10,
    include = ["/api/**"],
    exclude = ["/api/login/do_login", "/api/login/forget", "/api/common/global", "/api/install/**", "/api/share/**"]
)]
pub async fn auth_login(mut req: Request, next: Next) -> Result<Response, BizError> {
    let addr = RemoteAddr::from_request(&req);
    let Some(login_user) = find_login_user(req.headers(), addr.as_ref()).await? else {
        return Err(BizError::unauthorized());
    };
    req.extensions_mut().insert(login_user);
    Ok(next.run(req).await)
}

/// 通过请求头里的登录票据获取当前登录的用户,没有登录或者用户已经被禁用时返回None
/// - addr 客户端地址,用于记录操作日志
pub async fn find_login_user(headers: &HeaderMap, addr: Option<&RemoteAddr>) -> Result<Option<LoginUser>, BizError> {
    let Some(token) = request_util::get_token(headers) else {
        return Ok(None);
    };
    if token.is_empty() {
        //token为空,说明没有登录
        return Ok(None);
    }
    let mut ctx = lib_db::get_context();

    //先从登录记录里查找,找不到时再当作API票据查找
    let (user_id, client_flag) = match user_token_dao::select_by_token(&mut ctx, token.to_string()).await? {
        Some(it) => (it.userId, it.clientFlag),
        None => match user_dao::select_id_by_api_token(&mut ctx, token).await? {
            Some(it) => (it, -1),
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
    if user.parentId != 0 {
        //子账户:所属的主账户被禁用时,子账户也不能使用
        let Ok(parent) = user_dao::select_one(&mut ctx, user.parentId).await else {
            return Ok(None);
        };
        if parent.state != 1 {
            return Ok(None);
        }
    }
    let admin_id = user_dao::select_admin_id(&mut ctx).await?;
    Ok(Some(LoginUser {
        id: user_id,
        name: user.name.unwrap_or_default(),
        owner_id: if user.parentId == 0 { user_id } else { user.parentId },
        is_admin: admin_id == Some(user_id),
        is_sub: user.parentId != 0,
        ip: addr.map(|it| request_util::get_ip(headers, it)).unwrap_or_default(),
        client_flag,
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

/// 子账户不能使用的功能:子账户管理、WEB访问路径前缀、API票据、端对端加密密钥
#[FILTER(
    order = 20,
    include = ["/api/sub_account/**", "/api/mine/url_path", "/api/mine/api_token", "/api/mine/encryption_key"]
)]
pub async fn auth_main(req: Request, next: Next) -> Result<Response, BizError> {
    let is_sub = req
        .extensions()
        .get::<LoginUser>()
        .is_none_or(|it| it.is_sub);
    if is_sub {
        return Err(BizError::forbidden());
    }
    Ok(next.run(req).await)
}
