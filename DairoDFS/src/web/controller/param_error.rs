use crate::web::biz_error::BizError;
use hyperoute_core::{ParamErrorKind, ParamErrors};
use hyperoute_macro::PARAM_ERROR;

/// 请求参数解析失败时，转换成前端约定的字段错误格式
#[PARAM_ERROR]
pub fn on_param_error(errors: ParamErrors) -> BizError {
    let e = errors.first();
    let Some(param) = &e.param else {
        //针对整个请求的错误，比如请求体格式错误
        return BizError::msg(format!("请求数据格式错误: {}", e));
    };
    let reason = e.reason.as_deref().unwrap_or_default();
    let msg = match e.kind {
        ParamErrorKind::Missing => "该字段必须",
        ParamErrorKind::OutOfRange => "数值超出范围",
        _ if reason.ends_with("cannot parse integer from empty string") => "不能将空字符串转换为数字",
        _ if reason.ends_with("invalid digit found in string") => "无效的数字格式",
        _ => "类型转换失败",
    };
    BizError::param(param, msg)
}
