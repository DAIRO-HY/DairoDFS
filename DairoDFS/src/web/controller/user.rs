use crate::dao::user_dao;
use crate::extension::number::NumberExtension;
use crate::extension::string::StringExtension;
use crate::service::file_handle_service::HandleOptions;
use crate::service::user_service;
use crate::util::media::video_util::VideoCodec;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, db};
use hyperoute_macro::{GET, POST};
use validator::Validate;

/// 用户列表
#[GET("/api/user/list")]
pub async fn list() -> Result<Vec<model::UserList>, BizError> {
    let list = user_dao::select_main_list(&mut lib_db::get_context())
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
            ..model::UserDetail::with_options(&HandleOptions::default())
        });
    }
    let Ok(user) = user_dao::select_one(&mut lib_db::get_context(), id).await else {
        return Err(biz_error!("用户不存在"));
    };
    if user.parentId != 0 {
        //子账户由所属的主账户管理
        return Err(biz_error!("用户不存在"));
    }
    Ok(model::UserDetail {
        id: user.id,
        name: user.name.clone().unwrap_or_default(),
        email: user.email.clone().unwrap_or_default(),
        state: user.state,
        date: user.date.date_format(),
        ..model::UserDetail::with_options(&HandleOptions::of(&user))
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

    //新增的用户没有传的项默认开启,已有的用户没有传的项不修改
    let current = if form.id == 0 {
        HandleOptions::default()
    } else {
        match user_dao::select_one(&mut ctx, form.id).await {
            Ok(it) if it.parentId == 0 => HandleOptions::of(&it),
            _ => return Err(biz_error!("用户不存在")),
        }
    };
    let options = handle_options(&form, current)?;
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
        user_service::add(&mut ctx, form.name, pwd.md5(), form.email, form.state, &options.unwrap_or_default()).await?;
    } else {
        if form.state != 1 && user_dao::select_admin_id(&mut ctx).await? == Some(form.id) {
            return Err(BizError::field("state", "不能禁用管理员"));
        }
        user_dao::update_info(&mut ctx, form.id, form.name, form.email, form.state).await?;
        if let Some(pwd) = form.pwd {
            //填写了密码时才修改密码
            user_dao::set_pwd(&mut ctx, form.id, pwd.md5()).await?;
        }
        if let Some(options) = options {
            options.save(&mut ctx, form.id).await?;
        }
    }
    db::commit(ctx).await?;
    Ok(())
}

/// 表单里的上传文件处理设置
/// - 传了新的设置项时,没有传的项保持current里的值
/// - 只传了老版本的handleFile时,1:全部处理 0:全部不处理
/// - 都没有传时返回None(老版本的页面,不修改)
fn handle_options(form: &form::UserEdit, current: HandleOptions) -> Result<Option<HandleOptions>, BizError> {
    let thumb = parse_switch("handleThumb", form.handle_thumb)?;
    let smooth_video = parse_switch("handleSmoothVideo", form.handle_smooth_video)?;
    let compat_video = parse_switch("handleCompatVideo", form.handle_compat_video)?;
    let video_codec = match &form.video_codec {
        None => None,
        Some(it) => Some(VideoCodec::parse(it).ok_or_else(|| BizError::field("videoCodec", "视频编码设置不正确"))?),
    };
    if thumb.is_none() && smooth_video.is_none() && compat_video.is_none() && video_codec.is_none() {
        let Some(handle_file) = parse_switch("handleFile", form.handle_file)? else {
            return Ok(None);
        };
        return Ok(Some(HandleOptions::from_legacy(handle_file, current.video_codec)));
    }
    Ok(Some(HandleOptions {
        thumb: thumb.unwrap_or(current.thumb),
        smooth_video: smooth_video.unwrap_or(current.smooth_video),
        compat_video: compat_video.unwrap_or(current.compat_video),
        video_codec: video_codec.unwrap_or(current.video_codec),
    }))
}

/// 开关参数 0:关闭 1:开启
fn parse_switch(name: &str, value: Option<i64>) -> Result<Option<bool>, BizError> {
    match value {
        None => Ok(None),
        Some(0) => Ok(Some(false)),
        Some(1) => Ok(Some(true)),
        Some(_) => Err(BizError::field(name, "只能是0或1")),
    }
}

pub mod model {
    use crate::service::file_handle_service::HandleOptions;
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

        // 生成缩略图、预览图 0:否 1:是
        pub handle_thumb: i64,

        // 生成流畅版视频 0:否 1:是
        pub handle_smooth_video: i64,

        // 转换手机原生播放不了的视频 0:否 1:是
        pub handle_compat_video: i64,

        // 转码使用的视频编码 h264 h265
        pub video_codec: String,
    }

    impl UserDetail {

        /// 只填写了上传文件处理设置的详情
        pub fn with_options(options: &HandleOptions) -> Self {
            Self {
                handle_thumb: options.thumb as i64,
                handle_smooth_video: options.smooth_video as i64,
                handle_compat_video: options.compat_video as i64,
                video_codec: options.video_codec.name().to_string(),
                ..Default::default()
            }
        }
    }
}
