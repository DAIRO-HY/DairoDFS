use crate::web::biz_error::BizError;
use flate2::Compression;
use flate2::write::GzEncoder;
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Bytes};
use hyper::header::{ACCEPT_ENCODING, ACCEPT_RANGES, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, HeaderValue, VARY};
use hyper::StatusCode;
use hyperoute_core::{GZIP_MIN_SIZE, Next, Request, Response};
use hyperoute_macro::FILTER;
use std::io::Write;

/// 超过这个大小的响应不压缩,避免占用太多内存
const MAX_SIZE: u64 = 64 * 1024 * 1024;

/// 接口返回的JSON数据压缩(比如5万个文件的文件夹列表约5.8MB,压缩后只有几百KB)
/// 只压缩已经完整生成的JSON响应,文件下载(带Accept-Ranges)、Range请求、流式响应不处理
#[FILTER(order = 1, include = ["/api/**"])]
pub async fn compress(req: Request, next: Next) -> Result<Response, BizError> {
    let accept_gzip = req
        .headers()
        .get(ACCEPT_ENCODING)
        .and_then(|it| it.to_str().ok())
        .is_some_and(accept_gzip);
    let res = next.run(req).await;
    if !accept_gzip || !compressible(&res) {
        return Ok(res);
    }
    let (mut parts, body) = res.into_parts();
    let Ok(data) = body.collect().await.map(|it| it.to_bytes()) else {
        return Ok(hyperoute_core::internal_error_response());
    };

    //压缩比较耗CPU,放到阻塞线程里执行,不占用异步线程
    let gzip = tokio::task::spawn_blocking(move || {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data)?;
        encoder.finish()
    })
    .await;
    let Ok(Ok(gzip)) = gzip else {
        return Ok(hyperoute_core::internal_error_response());
    };
    parts.headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    parts.headers.insert(VARY, HeaderValue::from_static("Accept-Encoding"));
    parts.headers.insert(CONTENT_LENGTH, HeaderValue::from(gzip.len()));
    Ok(Response::from_parts(parts, hyperoute_core::boxed_full(Full::new(Bytes::from(gzip)))))
}

/// 客户端是否支持gzip,比如 Accept-Encoding: gzip, deflate, br
fn accept_gzip(value: &str) -> bool {
    value.split(',').any(|it| {
        let mut items = it.split(';');
        let name = items.next().unwrap_or("").trim();

        //q=0表示不接受
        let refused = items.any(|it| matches!(it.trim(), "q=0" | "q=0.0" | "q=0.00" | "q=0.000"));
        (name.eq_ignore_ascii_case("gzip") || name == "*") && !refused
    })
}

/// 响应是否需要压缩
fn compressible(res: &Response) -> bool {
    if res.status() != StatusCode::OK {
        return false;
    }
    let headers = res.headers();
    if headers.contains_key(CONTENT_ENCODING) || headers.contains_key(ACCEPT_RANGES) || headers.contains_key(CONTENT_RANGE) {
        return false;
    }
    let is_json = headers
        .get(CONTENT_TYPE)
        .and_then(|it| it.to_str().ok())
        .is_some_and(|it| it.starts_with("application/json"));
    if !is_json {
        return false;
    }

    //只处理已知大小的响应,太小的压缩没有意义
    res.body()
        .size_hint()
        .exact()
        .is_some_and(|it| (GZIP_MIN_SIZE..=MAX_SIZE).contains(&it))
}
