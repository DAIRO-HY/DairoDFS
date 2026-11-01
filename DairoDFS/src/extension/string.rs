
/// 将数字转换为更友好的数据大小格式
pub trait StringExtension {
    fn md5(&self) -> String;
}
impl<T: AsRef<str>> StringExtension for T {
    fn md5(&self) -> String {
        let digest = md5::compute(self.as_ref());

        // 转成16进制字符串
        format!("{:x}", digest)
    }
}