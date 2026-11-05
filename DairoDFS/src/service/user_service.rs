use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::user_dao::User;
use crate::dao::{dfs_file_dao, user_dao, user_token_dao};
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

/// 添加一个子账户,子账户没有自己的根目录,使用主账户授权的文件夹
/// - pwd 已经做过MD5的密码
pub async fn add_sub(
    ctx: &mut DbContext,
    parent_id: i64,
    name: String,
    pwd: String,
    nickname: Option<String>,
    state: i64,
) -> Result<i64, sqlx::Error> {
    let user = User {
        id: id_util::id(),
        name: Some(name),
        pwd: Some(pwd),
        state,
        date: time_util::current_millis(),
        parentId: parent_id,
        nickname,
        ..Default::default()
    };
    user_dao::insert(&mut *ctx, &user).await?;
    Ok(user.id)
}

/// 让用户在所有设备上退出登录(修改密码、禁用、删除时)
/// 按登录票据一条一条地删除,同步到其他服务器时删除的一定是同样的记录
pub async fn logout_all(ctx: &mut DbContext, user_id: i64) -> Result<(), sqlx::Error> {
    for it in user_token_dao::select_by_user_id(&mut *ctx, user_id).await? {
        user_token_dao::delete_by_token(&mut *ctx, it.token).await?;
    }
    Ok(())
}

/// 删除用户在某台设备上的登录记录(同一台设备只保留一条登录记录)
/// 只删除本机现有的记录:同一台设备在另一台服务器上刚刚登录的记录不受影响,两台服务器上的登录都有效
pub async fn logout_device(ctx: &mut DbContext, user_id: i64, device_id: String) -> Result<(), sqlx::Error> {
    for it in user_token_dao::select_by_user_id_and_device_id(&mut *ctx, user_id, device_id).await? {
        user_token_dao::delete_by_token(&mut *ctx, it.token).await?;
    }
    Ok(())
}
