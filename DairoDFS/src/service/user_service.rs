use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::user_dao::User;
use crate::dao::{dfs_file_dao, user_dao};
use crate::util::{id_util, time_util};
use sqlx_context::DbContext;

/// 添加一个用户,同时为用户创建一个根目录
/// - pwd 已经做过MD5的密码
pub async fn add(
    ctx: &mut DbContext,
    name: String,
    pwd: String,
    email: Option<String>,
    state: i64,
) -> Result<i64, sqlx::Error> {
    let user = User {
        id: id_util::id(),
        name: Some(name),
        pwd: Some(pwd),
        email,
        state,
        date: time_util::current_millis(),
        ..Default::default()
    };
    user_dao::insert(&mut *ctx, &user).await?;

    //创建用户时，为用户添加一个根目录(父目录ID为0,名称为空)
    let root_folder = DfsFile {
        id: id_util::id(),
        userId: user.id,
        date: time_util::current_millis(),
        ..Default::default()
    };
    dfs_file_dao::insert(&mut *ctx, &root_folder).await?;
    Ok(user.id)
}
