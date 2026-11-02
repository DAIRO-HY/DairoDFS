use crate::dao::{sub_account_grant_dao, user_dao, user_token_dao};
use crate::extension::number::NumberExtension;
use crate::extension::string::StringExtension;
use crate::service::operate_log_service::{self, op};
use crate::service::sub_account_service::{self, perm};
use crate::service::{dfs_file_service, user_service};
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::web::form;
use crate::{biz_errorf, db};
use hyperoute_core::Ext;
use hyperoute_macro::{DELETE, GET, POST};

/// 子账户列表
#[GET("/api/sub_account/list")]
pub async fn list(login: Ext<LoginUser>) -> Result<Vec<model::SubAccountList>, BizError> {
    let mut ctx = lib_db::get_context();
    let mut list = Vec::new();
    for it in user_dao::select_sub_list(&mut ctx, login.id).await? {
        let grant_count = sub_account_grant_dao::select_by_sub_user_id(&mut ctx, it.id).await?.len();
        list.push(model::SubAccountList {
            id: it.id,
            name: it.name.unwrap_or_default(),
            nickname: it.nickname.unwrap_or_default(),
            avatar: it.avatar.unwrap_or_default(),
            state: it.state,
            date: it.date.date_format(),
            grant_count,
        });
    }
    Ok(list)
}

/// 子账户详情
/// - id 子账户ID,0表示新增
#[GET("/api/sub_account/detail")]
pub async fn detail(login: Ext<LoginUser>, id: i64) -> Result<model::SubAccountDetail, BizError> {
    if id == 0 {
        return Ok(model::SubAccountDetail {
            state: 1,
            ..Default::default()
        });
    }
    let mut ctx = lib_db::get_context();
    let user = sub_account_service::select_own(&mut ctx, login.id, id).await?;
    let mut grants = Vec::new();
    for it in sub_account_grant_dao::select_by_sub_user_id(&mut ctx, id).await? {
        //授权的文件夹已经被删除时不显示,保存时会一起清除
        let Ok(folder) = dfs_file_service::select_own(&mut ctx, login.id, it.folderId).await else {
            continue;
        };
        if folder.deleteDate.is_some() {
            continue;
        }
        grants.push(model::Grant {
            folder_id: it.folderId,
            path: dfs_file_service::get_path_by_id(&mut ctx, it.folderId).await?,
            perms: it.perms,
        });
    }
    Ok(model::SubAccountDetail {
        id: user.id,
        name: user.name.unwrap_or_default(),
        nickname: user.nickname.unwrap_or_default(),
        avatar: user.avatar.unwrap_or_default(),
        state: user.state,
        date: user.date.date_format(),
        grants,
    })
}

/// 添加或修改子账户
/// - id 子账户ID,0表示新增
/// - pwd 密码,修改时为空表示不修改
/// - paths 授权的文件夹路径,和perms一一对应
/// - perms 权限,可以组合 1:浏览 2:打开/下载 4:上传 8:修改 16:删除 32:彻底删除 64:分享
#[POST("/api/sub_account/edit")]
pub async fn edit(
    login: Ext<LoginUser>,
    id: i64,
    name: String,
    pwd: Option<String>,
    nickname: Option<String>,
    state: i64,
    paths: Vec<String>,
    perms: Vec<i64>,
) -> Result<i64, BizError> {
    let name = name.trim().to_string();
    let name_len = name.chars().count();
    if !(2..=32).contains(&name_len) {
        return Err(BizError::field("name", "用户名长度必须在2到32位之间"));
    }
    let pwd = form::none_if_empty(pwd);
    if let Some(pwd) = &pwd {
        sub_account_service::check_pwd(pwd)?;
    }
    let nickname = form::none_if_empty(nickname);
    if nickname.as_ref().is_some_and(|it| it.chars().count() > 32) {
        return Err(BizError::field("nickname", "昵称长度不能超过32位"));
    }
    if state != 0 && state != 1 {
        return Err(BizError::field("state", "用户状态不正确"));
    }
    if paths.len() != perms.len() {
        return Err(BizError::param("perms", "授权的文件夹和权限数量不一致"));
    }

    let mut ctx = db::begin().await?;
    if let Some(it) = user_dao::select_by_name(&mut ctx, name.clone()).await? {
        if it.id != id {
            return Err(BizError::field("name", "用户名已经存在"));
        }
    }

    //授权的文件夹必须是主账户已经存在的文件夹,同一个文件夹只保留一条
    let mut grants: Vec<(i64, i64)> = Vec::new();
    let mut grant_log = Vec::new();
    for (path, perms) in paths.iter().zip(perms) {
        let Some(folder_id) = dfs_file_service::get_folder_id(&mut ctx, login.id, path, false).await? else {
            return Err(biz_errorf!("文件夹不存在:{}", path));
        };
        let perms = sub_account_service::normalize_perms(perms);
        if perms == 0 {
            return Err(biz_errorf!("没有选择权限:{}", path));
        }
        grant_log.push(serde_json::json!({"path": path, "perms": perms}));
        match grants.iter_mut().find(|it| it.0 == folder_id) {
            Some(it) => it.1 |= perms,
            None => grants.push((folder_id, perms)),
        }
    }

    let op_type = if id == 0 { op::SUB_ACCOUNT_ADD } else { op::SUB_ACCOUNT_EDIT };
    let detail = serde_json::json!({
        "name": name,
        "nickname": nickname,
        "state": state,
        "pwdChanged": pwd.is_some(),
        "grants": grant_log,
    });
    let id = if id == 0 {
        let Some(pwd) = pwd else {
            return Err(BizError::field("pwd", "密码必填"));
        };
        user_service::add_sub(&mut ctx, login.id, name, pwd.md5(), nickname, state).await?
    } else {
        sub_account_service::select_own(&mut ctx, login.id, id).await?;
        user_dao::update_sub(&mut ctx, id, name, nickname, state).await?;
        if let Some(pwd) = &pwd {
            user_dao::set_pwd(&mut ctx, id, pwd.md5()).await?;
        }
        if pwd.is_some() || state != 1 {
            //修改了密码或者禁用时,已经登录的设备需要重新登录
            user_token_dao::delete_by_user_id(&mut ctx, id).await?;
        }
        id
    };
    sub_account_service::set_grants(&mut ctx, id, &grants).await?;
    db::commit(ctx).await?;
    let mut detail = detail;
    detail["subId"] = id.into();
    operate_log_service::add(&login, op_type, None, None, Some(detail));
    Ok(id)
}

/// 删除子账户,已经上传的文件仍然属于主账户
#[DELETE("/api/sub_account/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    let mut names = Vec::new();
    for id in ids {
        let user = sub_account_service::select_own(&mut ctx, login.id, id).await?;
        sub_account_service::delete(&mut ctx, id).await?;
        names.push((id, user.name.unwrap_or_default()));
    }
    db::commit(ctx).await?;
    for (id, name) in names {
        operate_log_service::add(&login, op::SUB_ACCOUNT_DELETE, None, None, Some(serde_json::json!({"subId": id, "name": name})));
    }
    Ok(())
}

/// 权限的分类和说明,管理页面按分类显示
#[GET("/api/sub_account/perms")]
pub async fn perms() -> Vec<model::PermGroup> {
    let item = |value: i64, name: &str, desc: &str| model::PermItem {
        value,
        name: name.to_string(),
        desc: desc.to_string(),
    };
    vec![
        model::PermGroup {
            name: "查看".to_string(),
            list: vec![
                item(perm::VIEW, "浏览", "查看文件列表、属性、缩略图、相册"),
                item(perm::OPEN, "打开/下载", "预览、播放、下载原文件"),
            ],
        },
        model::PermGroup {
            name: "编辑".to_string(),
            list: vec![
                item(perm::UPLOAD, "上传", "上传文件、新建文件夹"),
                item(perm::MODIFY, "修改", "重命名、移动、复制、修改文件类型"),
            ],
        },
        model::PermGroup {
            name: "删除".to_string(),
            list: vec![
                item(perm::DELETE, "删除", "删除到回收站、从回收站恢复"),
                item(perm::PURGE, "彻底删除", "从回收站彻底删除"),
            ],
        },
        model::PermGroup {
            name: "分享".to_string(),
            list: vec![item(perm::SHARE, "分享", "创建分享链接、管理自己的分享")],
        },
    ]
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubAccountList {

        // 子账户ID
        pub id: i64,

        // 用户名
        pub name: String,

        // 昵称
        pub nickname: String,

        // 头像的版本,为空表示没有头像,通过 /api/avatar/{用户ID} 获取
        pub avatar: String,

        // 状态 0:禁用 1:正常
        pub state: i64,

        // 创建日期
        pub date: String,

        // 授权的文件夹数量
        pub grant_count: usize,
    }

    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubAccountDetail {

        // 子账户ID
        pub id: i64,

        // 用户名
        pub name: String,

        // 昵称
        pub nickname: String,

        // 头像的版本,为空表示没有头像,通过 /api/avatar/{用户ID} 获取
        pub avatar: String,

        // 状态 0:禁用 1:正常
        pub state: i64,

        // 创建日期
        pub date: String,

        // 授权的文件夹
        pub grants: Vec<Grant>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Grant {

        // 文件夹ID
        pub folder_id: i64,

        // 文件夹路径
        pub path: String,

        // 权限
        pub perms: i64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PermGroup {

        // 分类名称
        pub name: String,

        // 分类下的权限
        pub list: Vec<PermItem>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PermItem {

        // 权限值
        pub value: i64,

        // 权限名称
        pub name: String,

        // 说明
        pub desc: String,
    }
}
