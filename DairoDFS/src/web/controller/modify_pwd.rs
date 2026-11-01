use crate::dao::{user_dao, user_token_dao};
use crate::db;
use crate::extension::string::StringExtension;
use crate::util::request_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::web::form;
use hyperoute_core::{Ext, Raw};
use hyperoute_macro::PUT;
use validator::Validate;

/// 修改密码,修改成功后所有客户端都需要重新登录
#[PUT("/api/modify_pwd")]
pub async fn modify(login: Ext<LoginUser>, form: form::ModifyPwd) -> Result<Raw, BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    let mut ctx = db::begin().await?;
    let user = user_dao::select_one(&mut ctx, login.id).await?;
    if user.pwd.as_deref() != Some(form.old_pwd.md5().as_str()) {
        return Err(BizError::field("oldPwd", "旧密码不正确"));
    }
    user_dao::set_pwd(&mut ctx, login.id, form.pwd.md5()).await?;
    user_token_dao::delete_by_user_id(&mut ctx, login.id).await?;
    db::commit(ctx).await?;
    Ok(Raw::new("").header("set-cookie", request_util::clear_token_cookie()))
}
