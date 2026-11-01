use crate::system_config;
use crate::web::biz_error::BizError;
use crate::biz_error;
use hyperoute_core::{Next, Request, Response};
use hyperoute_macro::FILTER;

/// 分布式同步验证,路径格式: /distributed/{主机端的同步票据}/{分机端的同步票据}/...
#[FILTER(order = 5, include = ["/distributed/**"])]
pub async fn distributed_validate(req: Request, next: Next) -> Result<Response, BizError> {
    let paths: Vec<&str> = req.uri().path().split('/').collect();
    if paths.len() < 5 {
        return Err(BizError::not_found());
    }
    let (master_token, client_token) = (paths[2], paths[3]);
    if master_token == client_token {
        return Err(biz_error!("禁止本机循环同步"));
    }
    if master_token != system_config::get().distributed_token {
        return Err(biz_error!("票据无效"));
    }
    Ok(next.run(req).await)
}
