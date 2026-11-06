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

/// 请求的数据范围(Range请求头)
#[derive(Debug, PartialEq)]
pub enum ByteRange {

    // 返回全部数据
    Full,

    // 返回其中一段,开始和结束位置都包含在内
    Partial(u64, u64),

    // 范围不合法(超过数据大小),返回416
    Unsatisfiable,
}

/// 是否和客户端缓存的数据一致(If-None-Match),一致时返回304
pub fn not_modified(headers: &HeaderMap, etag: &str) -> bool {
    let Some(value) = headers.get("if-none-match").and_then(|it| it.to_str().ok()) else {
        return false;
    };
    value.split(',').map(str::trim).any(|tag| tag == "*" || tag.strip_prefix("W/").unwrap_or(tag) == etag)
}

/// 获取请求的数据范围,支持断点续传
/// If-Range和ETag不一致时(数据已经变了),返回全部数据;只支持单段,写法不认识或多段时忽略Range
/// - size 数据大小
/// - etag 数据的ETag(带引号)
pub fn byte_range(headers: &HeaderMap, size: u64, etag: &str) -> ByteRange {
    let Some(range) = headers.get("range").and_then(|it| it.to_str().ok()) else {
        return ByteRange::Full;
    };
    if let Some(if_range) = headers.get("if-range").and_then(|it| it.to_str().ok()) {
        //强比较,弱ETag和日期都不算一致
        if if_range.trim() != etag {
            return ByteRange::Full;
        }
    }
    parse_range(range, size)
}

/// 解析Range: bytes=a-b / bytes=a- / bytes=-n
fn parse_range(value: &str, size: u64) -> ByteRange {
    let value = value.trim();
    let Some(spec) = value.get(..6).filter(|it| it.eq_ignore_ascii_case("bytes=")).map(|_| &value[6..]) else {
        return ByteRange::Full;
    };
    if spec.contains(',') {
        return ByteRange::Full;
    }
    let Some((first, last)) = spec.trim().split_once('-') else {
        return ByteRange::Full;
    };
    let (first, last) = (first.trim(), last.trim());
    let is_digits = |it: &str| !it.is_empty() && it.bytes().all(|b| b.is_ascii_digit());
    if first.is_empty() {
        //最后n个字节
        if !is_digits(last) {
            return ByteRange::Full;
        }
        let n: u64 = last.parse().unwrap_or(u64::MAX);
        if n == 0 || size == 0 {
            return ByteRange::Unsatisfiable;
        }
        return ByteRange::Partial(size.saturating_sub(n), size - 1);
    }
    if !is_digits(first) || !(last.is_empty() || is_digits(last)) {
        return ByteRange::Full;
    }
    let Ok(start) = first.parse::<u64>() else {
        return ByteRange::Unsatisfiable;
    };
    let end = if last.is_empty() { u64::MAX } else { last.parse::<u64>().unwrap_or(u64::MAX) };
    if end < start {
        return ByteRange::Full;
    }
    if start >= size {
        return ByteRange::Unsatisfiable;
    }
    ByteRange::Partial(start, end.min(size - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(list: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in list {
            map.insert(*name, value.parse().unwrap());
        }
        map
    }

    #[test]
    fn range() {
        let etag = "\"abc\"";
        assert_eq!(byte_range(&headers(&[]), 1000, etag), ByteRange::Full);
        assert_eq!(byte_range(&headers(&[("range", "bytes=0-99")]), 1000, etag), ByteRange::Partial(0, 99));
        assert_eq!(byte_range(&headers(&[("range", "bytes=900-")]), 1000, etag), ByteRange::Partial(900, 999));
        assert_eq!(byte_range(&headers(&[("range", "bytes=900-5000")]), 1000, etag), ByteRange::Partial(900, 999));
        assert_eq!(byte_range(&headers(&[("range", "bytes=-100")]), 1000, etag), ByteRange::Partial(900, 999));
        assert_eq!(byte_range(&headers(&[("range", "bytes=1000-")]), 1000, etag), ByteRange::Unsatisfiable);
        assert_eq!(byte_range(&headers(&[("range", "bytes=0-1,5-9")]), 1000, etag), ByteRange::Full);
        assert_eq!(byte_range(&headers(&[("range", "items=0-9")]), 1000, etag), ByteRange::Full);
        assert_eq!(byte_range(&headers(&[("range", "bytes=9-0")]), 1000, etag), ByteRange::Full);

        //If-Range一致时返回一段,不一致时返回全部
        assert_eq!(byte_range(&headers(&[("range", "bytes=10-"), ("if-range", "\"abc\"")]), 1000, etag), ByteRange::Partial(10, 999));
        assert_eq!(byte_range(&headers(&[("range", "bytes=10-"), ("if-range", "\"old\"")]), 1000, etag), ByteRange::Full);
        assert_eq!(byte_range(&headers(&[("range", "bytes=10-"), ("if-range", "W/\"abc\"")]), 1000, etag), ByteRange::Full);
        assert_eq!(byte_range(&headers(&[("range", "bytes=10-"), ("if-range", "Wed, 21 Oct 2026 07:28:00 GMT")]), 1000, etag), ByteRange::Full);

        assert!(not_modified(&headers(&[("if-none-match", "\"abc\"")]), etag));
        assert!(not_modified(&headers(&[("if-none-match", "W/\"abc\", \"x\"")]), etag));
        assert!(!not_modified(&headers(&[("if-none-match", "\"old\"")]), etag));
    }
}
