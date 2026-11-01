use crate::service::sync;
use crate::service::sync::sync_info::SyncInfoView;
use hyperoute_core::{Message, WebSocket, WebSocketError};
use hyperoute_macro::{GET, POST, WEBSOCKET};
use std::time::Duration;

/// 获取同步主机的状态
#[GET("/api/sync/list")]
pub async fn list() -> Vec<SyncInfoView> {
    sync::sync_info::views()
}

/// 重新开始日志同步
#[POST("/api/sync/by_log")]
pub async fn by_log() {
    sync::listen_all();
}

/// 全量同步(在后台执行)
#[POST("/api/sync/by_table")]
pub async fn by_table() {
    tokio::spawn(sync::sync_by_table::sync_all());
}

/// 实时推送同步主机的状态,有变化时才推送
#[WEBSOCKET("/api/sync/info")]
pub async fn info(mut socket: WebSocket) -> Result<(), WebSocketError> {
    let mut pre: Vec<SyncInfoView> = Vec::new();
    loop {
        let views = sync::sync_info::views();
        if views != pre {
            socket.send(Message::text(serde_json::to_string(&views).unwrap())).await?;
            pre = views;
        }

        //同时监听客户端的消息,客户端关闭连接时结束
        tokio::select! {
            message = socket.recv() => match message {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return Ok(()),
                _ => {}
            },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        }
    }
}
