use hyper::HeaderMap;
use hyperoute_core::RemoteAddr;

/// 登录票据的Cookie名
pub const TOKEN_COOKIE: &str = "token";

/// 获取客户端IP,经过代理时优先使用代理转发的IP
pub fn get_ip(headers: &HeaderMap, addr: &RemoteAddr) -> String {
    for name in ["x-forwarded-for", "proxy-client-ip", "wl-proxy-client-ip"] {
        if let Some(ip) = headers.get(name).and_then(|it| it.to_str().ok()) {
            //经过多层代理时,第一个才是客户端的IP
            let ip = ip.split(',').next().unwrap_or_default().trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }
    addr.ip().to_string()
}

/// 获取登录票据
/// 网页端从Cookie获取,APP或API调用从请求头 Authorization: Bearer <token> 获取
pub fn get_token(headers: &HeaderMap) -> Option<&str> {
    let bearer = headers
        .get("authorization")
        .and_then(|it| it.to_str().ok())
        .and_then(|it| it.strip_prefix("Bearer "))
        .map(|it| it.trim());
    if let Some(token) = bearer {
        return Some(token);
    }
    get_cookie(headers, TOKEN_COOKIE)
}

/// 从请求头中获取指定名称的Cookie值
pub fn get_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all("cookie")
        .iter()
        .filter_map(|it| it.to_str().ok())
        .flat_map(|it| it.split(';'))
        .filter_map(|it| it.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// 设置登录票据的Cookie(HttpOnly防止JS读取)
pub fn token_cookie(token: &str) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=315360000",
        TOKEN_COOKIE, token
    )
}

/// 清除登录票据的Cookie
pub fn clear_token_cookie() -> String {
    format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0", TOKEN_COOKIE)
}
