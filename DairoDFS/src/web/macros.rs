
/// 用法：
///   return Err(biz_error!("message"));
/// 展开为：
///   BizError::msg("message")
#[macro_export]
macro_rules! biz_error {
    // String和&str都支持,如果是String零拷贝，如果是&str则转换为String
    ($msg:expr) => {
        $crate::web::biz_error::BizError::msg($msg)
    };
}

#[macro_export]
macro_rules! biz_errorf {
    // 可变参数格式化
    ($($arg:tt)*) => {
        $crate::web::biz_error::BizError::msg(std::format!($($arg)*))
    };
}
