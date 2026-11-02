use crate::dao::dfs_file_dao;
use crate::web::controller::files;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use hyperoute_core::Ext;
use hyperoute_macro::GET;

/// 获取子文件夹名列表(选择文件夹用)
/// - folder 文件夹路径,空字符串代表根目录
#[GET("/api/folder_selector/list")]
pub async fn list(login: Ext<LoginUser>, folder: String) -> Result<Vec<String>, BizError> {
    let mut ctx = lib_db::get_context();
    let (scope, folder_id, folder_perms) = files::list_folder(&mut ctx, &login, &folder).await?;
    let list = dfs_file_dao::select_sub_list(&mut ctx, login.owner_id, folder_id)
        .await?
        .into_iter()
        .filter(|it| it.storageId == 0 && scope.visible(folder_perms, it.id))
        .map(|it| it.name)
        .collect();
    Ok(list)
}
