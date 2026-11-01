use crate::web::model::ResultData;
use hyper::StatusCode;
use hyperoute_core::IntoErrorResponse;
use std::collections::HashMap;

/// 业务错误，handler返回Err时转换成错误响应
/// 前端约定(api-http.js)：
/// - 401：跳转到登录页
/// - JSON且code=2：表单字段错误，data为 字段名 -> 错误消息列表
/// - 其他：直接显示响应文本
#[derive(Debug)]
pub struct BizError {
    status: StatusCode,
    body: String,
    is_json: bool,
}

impl BizError {
    /// 业务错误消息，状态码为500
    pub fn msg(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: msg.into(),
            is_json: false,
        }
    }

    /// 未登录或登录已过期
    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: String::new(),
            is_json: false,
        }
    }

    /// 没有权限(已经登录,但不是管理员)
    pub fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            body: "权限不足".to_string(),
            is_json: false,
        }
    }

    /// 资源正在被使用(例如同一个文件正在上传),客户端稍后重试,状态码为409
    pub fn busy(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: msg.into(),
            is_json: false,
        }
    }

    /// 文件下载时,文件不存在
    pub fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: String::new(),
            is_json: false,
        }
    }

    /// 单个字段错误，状态码为500
    pub fn field(field: &str, msg: &str) -> Self {
        Self::field_map(
            StatusCode::INTERNAL_SERVER_ERROR,
            HashMap::from([(field.to_string(), vec![msg.to_string()])]),
        )
    }

    /// 将表单验证错误转换为字段错误，状态码为500
    pub fn fields(errors: validator::ValidationErrors) -> Self {
        let field_error_map: HashMap<String, Vec<String>> = errors
            .field_errors()
            .into_iter()
            .map(|(field, errs)| {
                (
                    //表单字段名是驼峰命名(serde rename_all = "camelCase"),验证错误里的是Rust字段名,转换后前端才能对应到输入框
                    to_camel_case(&field),
                    errs.iter()
                        .filter_map(|err| err.message.as_ref().map(|it| it.to_string()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        Self::field_map(StatusCode::INTERNAL_SERVER_ERROR, field_error_map)
    }

    /// 请求参数错误，状态码为400
    pub fn param(field: &str, msg: &str) -> Self {
        Self::field_map(
            StatusCode::BAD_REQUEST,
            HashMap::from([(field.to_string(), vec![msg.to_string()])]),
        )
    }

    fn field_map(status: StatusCode, data: HashMap<String, Vec<String>>) -> Self {
        let result = ResultData {
            code: 2,
            msg: "表单验证失败".to_string(),
            data,
        };
        Self {
            status,
            body: serde_json::to_string(&result).unwrap(),
            is_json: true,
        }
    }
}

/// 数据库操作失败时直接用 ? 返回,错误详情只记录日志,不返回给前端
impl From<sqlx::Error> for BizError {
    fn from(err: sqlx::Error) -> Self {
        eprintln!("-->数据库操作失败: {}", err);
        BizError::msg("数据库操作失败")
    }
}

/// 文件操作失败时直接用 ? 返回
impl From<std::io::Error> for BizError {
    fn from(err: std::io::Error) -> Self {
        BizError::msg(format!("文件操作失败: {}", err))
    }
}

/// 将Rust字段名(snake_case)转换成驼峰命名
fn to_camel_case(field: &str) -> String {
    let mut result = String::with_capacity(field.len());
    let mut upper = false;
    for c in field.chars() {
        if c == '_' && !result.is_empty() {
            upper = true;
        } else if upper {
            result.extend(c.to_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// 后台任务里需要把错误信息记录下来
impl std::fmt::Display for BizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.body)
    }
}

impl IntoErrorResponse for BizError {
    fn status(&self) -> StatusCode {
        self.status
    }

    fn message(&self) -> String {
        self.body.clone()
    }

    fn headers(&self) -> Vec<(&'static str, String)> {
        let content_type = if self.is_json {
            "application/json"
        } else {
            "text/plain; charset=utf-8"
        };
        vec![("content-type", content_type.to_string())]
    }
}
