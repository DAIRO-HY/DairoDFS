use crate::application;
use crate::dao::user_dao;
use crate::system_config;
use crate::web::biz_error::BizError;
use hyperoute_macro::GET;

/// 获取全局数据API
#[GET("/api/common/global")]
pub async fn global() -> Result<model::GlobalData, BizError> {
    let user_count = user_dao::count(&mut lib_db::get_context()).await?;
    Ok(model::GlobalData {
        version: application::VERSION.to_string(),
        is_init: user_count > 0,
        cluster_id: system_config::get().cluster_id.clone(),
    })
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GlobalData {

        // 版本号
        pub version: String,

        // 是否已经初始化(已经创建了管理员)
        pub is_init: bool,

        // 集群ID,客户端缓存文件时用来区分文件来自哪个集群
        pub cluster_id: String,
    }
}
