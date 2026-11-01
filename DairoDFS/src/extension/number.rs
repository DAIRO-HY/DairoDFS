use chrono::DateTime;

pub trait NumberExtension {

    /// 将时间戳毫秒转换为日期格式
    fn date_format(&self) -> String;

    /// 转换为更友好的数据大小格式
    fn data_size(&self) -> String;

    /// 将其转换成时间单位
    fn time_format(&self) -> String;
}
pub trait NumberExtensionType: Copy + TryInto<u64> {}
impl NumberExtensionType for i8 {}
impl NumberExtensionType for i16 {}
impl NumberExtensionType for i32 {}
impl NumberExtensionType for i64 {}
impl NumberExtensionType for i128 {}
impl NumberExtensionType for u8 {}
impl NumberExtensionType for u16 {}
impl NumberExtensionType for u32 {}
impl NumberExtensionType for u64 {}
impl NumberExtensionType for u128 {}
impl NumberExtensionType for usize {}

impl<T> NumberExtension for T
where
    T: NumberExtensionType,
{
    /// 将时间戳毫秒转换为日期格式
    fn date_format(&self) -> String {
        let timestamp = (*self).try_into().unwrap_or(0);
        if timestamp == 0 {
            return String::new();
        }
        let dt = DateTime::from_timestamp_millis(timestamp as i64)
            .unwrap()
            .with_timezone(&chrono::Local);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// 转换为更友好的数据大小格式
    fn data_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        let value = (*self).try_into().unwrap_or(0);
        match value {
            b if b < KB => format!("{} Byte", b),
            b if b < MB => format!("{:.2} KB", b as f64 / KB as f64),
            b if b < GB => format!("{:.2} MB", b as f64 / MB as f64),
            b if b < TB => format!("{:.2} GB", b as f64 / GB as f64),
            b => format!("{:.2} TB", b as f64 / TB as f64),
        }
    }

    /// 将其转换成时间单位
    fn time_format(&self) -> String {
        let value = (*self).try_into().unwrap_or(0) / 1000;
        let hour = value / (60 * 60);//小时数
        let minu = value % (60 * 60) / 60;
        let secs = value % 60;//秒数
        if hour > 0{
            format!("{:02}:{:02}:{:02}",hour,minu,secs)
        }else{
            format!("{:02}:{:02}",minu,secs)
        }
    }
}

/// 将数字转换为更友好的数据大小格式
pub trait Div {
    fn div(&self, b: f64, digits: usize) -> String;
}
pub trait FromDivType: Copy + TryInto<f64> {}

impl FromDivType for f32 {}
impl FromDivType for f64 {}
impl FromDivType for i8 {}
impl FromDivType for i16 {}
impl FromDivType for i32 {}
impl FromDivType for u8 {}
impl FromDivType for u16 {}
impl FromDivType for u32 {}

impl<T> Div for T
where
    T: FromDivType,
{
    fn div(&self, b: f64, digits: usize) -> String {
        if b == 0.0 {
            return "NaN".to_string();
        }
        format!("{:.*}", digits, (*self).try_into().unwrap_or(0.0) / b)
    }
}

/// 将数字转换成较短的字母组合
pub trait ShortString {
    fn to_short_string(&self) -> String;
}
impl ShortString for i64 {
    fn to_short_string(&self) -> String {
        let chars = crate::application::SHORT_CHAR.as_bytes();
        let char_len = chars.len() as i64;
        let mut target = *self - 1;
        let mut short = Vec::new();
        loop {
            short.push(chars[(target % char_len) as usize]);
            target /= char_len;
            if target == 0 {
                break;
            }
        }
        short.reverse();
        String::from_utf8(short).unwrap()
    }
}
