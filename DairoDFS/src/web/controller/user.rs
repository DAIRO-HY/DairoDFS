use crate::dao::user_dao;
use crate::extension::number::NumberExtension;
use crate::extension::string::StringExtension;
use crate::service::user_service;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, db};
use hyperoute_macro::{GET, POST};
use validator::Validate;

/// 用户列表
#[GET("/api/user/list")]
pub async fn list() -> Result<Vec<model::UserList>, BizError> {
    let list = user_dao::select_all(&mut lib_db::get_context())
        .await?
        .into_iter()
        .map(|it| model::UserList {
            id: it.id,
            name: it.name.unwrap_or_default(),
            email: it.email.unwrap_or_default(),
            state: if it.state == 1 { "启用" } else { "禁用" }.to_string(),
            date: it.date.date_format(),
        })
        .collect();
    Ok(list)
}

/// 用户详情
#[GET("/api/user/detail")]
pub async fn detail(id: i64) -> Result<model::UserDetail, BizError> {
    if id == 0 {
        //新增用户
        return Ok(model::UserDetail {
            state: 1,
            ..Default::default()
        });
    }
    let Ok(user) = user_dao::select_one(&mut lib_db::get_context(), id).await else {
        return Err(biz_error!("用户不存在"));
    };
    Ok(model::UserDetail {
        id: user.id,
        name: user.name.unwrap_or_default(),
        email: user.email.unwrap_or_default(),
        state: user.state,
        date: user.date.date_format(),
    })
}

/// 添加或更新用户
#[POST("/api/user/edit")]
pub async fn edit(mut form: form::UserEdit) -> Result<(), BizError> {
    form.pwd = form::none_if_empty(form.pwd);
    form.email = form::none_if_empty(form.email);
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    if form.state != 0 && form.state != 1 {
        return Err(BizError::field("state", "用户状态不正确"));
    }
    let mut ctx = db::begin().await?;
    if let Some(it) = user_dao::select_by_name(&mut ctx, form.name.clone()).await? {
        if it.id != form.id {
            return Err(BizError::field("name", "用户名已经存在"));
        }
    }
    if let Some(email) = &form.email {
        if let Some(it) = user_dao::select_by_email(&mut ctx, email.clone()).await? {
            if it.id != form.id {
                return Err(BizError::field("email", "该邮箱已经被其他用户使用"));
            }
        }
    }
    if form.id == 0 {
        let Some(pwd) = form.pwd else {
            return Err(BizError::field("pwd", "密码必填"));
        };
        user_service::add(&mut ctx, form.name, pwd.md5(), form.email, form.state).await?;
    } else {
        if user_dao::select_one(&mut ctx, form.id).await.is_err() {
            return Err(biz_error!("用户不存在"));
        }
        if form.state != 1 && user_dao::select_admin_id(&mut ctx).await? == Some(form.id) {
            return Err(BizError::field("state", "不能禁用管理员"));
        }
        user_dao::update_info(&mut ctx, form.id, form.name, form.email, form.state).await?;
        if let Some(pwd) = form.pwd {
            //填写了密码时才修改密码
            user_dao::set_pwd(&mut ctx, form.id, pwd.md5()).await?;
        }
    }
    db::commit(ctx).await?;
    Ok(())
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UserList {

        // 用户ID
        pub id: i64,

        // 用户名
        pub name: String,

        // 邮箱
        pub email: String,

        // 用户状态
        pub state: String,

        // 创建日期
        pub date: String,
    }

    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UserDetail {

        // 用户ID
        pub id: i64,

        // 用户名
        pub name: String,

        // 邮箱
        pub email: String,

        // 用户状态 0:禁用 1:正常
        pub state: i64,

        // 创建日期
        pub date: String,
    }
}
