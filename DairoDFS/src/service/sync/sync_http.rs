use crate::service::sync::distributed_push::KEEP_ALIVE_TIME;
use std::sync::LazyLock;
use std::time::Duration;

/// 同步数据请求用的客户端
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap()
});

/// 监听日志的长连接客户端,主机每隔一段时间会发送心跳,超过这个时间没有收到数据就认为连接断开
pub static LISTEN_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .read_timeout(Duration::from_secs(KEEP_ALIVE_TIME + 10))
        .build()
        .unwrap()
});

/// 下载文件用的客户端(没有总超时时间,只限制读数据的间隔)
pub static DOWNLOAD_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .build()
        .unwrap()
});

/// 请求同步数据
pub async fn request(url: &str) -> Result<String, String> {
    let resp = CLIENT.get(url).send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Status: {}  Body:{}", status.as_u16(), body));
    }
    Ok(body)
}
