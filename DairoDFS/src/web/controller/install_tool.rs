use crate::dao::user_dao;
use crate::service::tool_install_service;
use crate::service::tool_install_service::InstallProgress;
use crate::web::biz_error::BizError;
use crate::web::filter::auth;
use hyper::HeaderMap;
use hyperoute_core::{Message, WebSocket, WebSocketError};
use hyperoute_macro::{GET, POST, WEBSOCKET};
use std::time::Duration;

/// 获取工具的安装状态
/// - name 工具名 ffmpeg,ffprobe,libraw,magick,exiftool
#[GET("/api/install/tool/{name}")]
pub async fn info(name: String, headers: &HeaderMap) -> Result<InstallProgress, BizError> {
    let tool = check(&name, headers).await?;
    Ok(tool_install_service::progress(tool).await)
}

/// 开始安装
#[POST("/api/install/tool/{name}/install")]
pub async fn install(name: String, headers: &HeaderMap) -> Result<(), BizError> {
    let tool = check(&name, headers).await?;
    tool_install_service::install(tool);
    Ok(())
}

/// 推送安装进度,安装结束后关闭连接
#[WEBSOCKET("/api/install/tool/{name}/progress")]
pub async fn progress(mut socket: WebSocket, name: String, headers: HeaderMap) -> Result<(), WebSocketError> {
    let Ok(tool) = check(&name, &headers).await else {
        return socket.close(None).await;
    };

    //记录上次发送的数据,两次一样时不发送
    let mut pre: Option<InstallProgress> = None;
    loop {
        let progress = tool_install_service::current_progress(tool);
        let is_running = progress.is_running;
        if pre.as_ref() != Some(&progress) {
            socket.send(Message::text(serde_json::to_string(&progress).unwrap())).await?;
            pre = Some(progress);
        }
        if !is_running {
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    socket.close(None).await
}

/// 检查工具名,系统初始化之后只有管理员才能安装
async fn check(name: &str, headers: &HeaderMap) -> Result<&'static str, BizError> {
    let Some(tool) = tool_install_service::tool_name(name) else {
        return Err(BizError::not_found());
    };
    if user_dao::count(&mut lib_db::get_context()).await? == 0 {
        return Ok(tool);
    }
    match auth::find_login_user(headers).await? {
        Some(it) if it.is_admin => Ok(tool),
        Some(_) => Err(BizError::forbidden()),
        None => Err(BizError::unauthorized()),
    }
}
