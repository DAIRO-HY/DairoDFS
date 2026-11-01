use crate::dao::user_dao;
use crate::service::dfs_file_service;
use crate::web::biz_error::BizError;
use crate::web::controller::files;
use hyper::HeaderMap;
use hyperoute_core::Stream;
use hyperoute_macro::GET;

/// 通过用户的WEB访问路径前缀公开下载文件(不需要登录)
/// - url_path 用户的WEB访问路径前缀
/// - path 文件路径
#[GET("/d/{url_path}/{*path}")]
pub async fn download(url_path: String, path: String, headers: &HeaderMap) -> Result<Stream, BizError> {
    let mut ctx = lib_db::get_context();
    let Some(user_id) = user_dao::select_id_by_url_path(&mut ctx, &url_path).await? else {
        return Err(BizError::not_found());
    };
    let Ok(Some(file)) = dfs_file_service::select_by_path(&mut ctx, user_id, &format!("/{}", path)).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 {
        return Err(BizError::not_found());
    }
    let stream = files::file_stream(&file, headers)
        .await?
        //允许在其他网站上引用
        .set_header("access-control-allow-origin", "*")
        //按路径访问,同一个路径的文件内容可能会改变,每次都要向服务器确认
        .set_header("cache-control", "public, no-cache");
    Ok(stream)
}
