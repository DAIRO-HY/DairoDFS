//! 账号的上传文件处理设置

use crate::dao::user_dao::{self, User};
use crate::util::media::video_util::VideoCodec;
use sqlx_context::DbContext;

/// 上传的文件要做哪些处理(子账户上传的文件属于主账户,按主账户的设置处理)
///
/// 文件属性(尺寸、拍摄时间、时长等)总是生成,不受这里的设置影响
/// 同一个文件在其他地方已经生成过的附属文件,不管设置是否开启都直接复用(不占空间,也不耗费CPU)
/// 修改设置只对之后上传的文件生效,已有的文件可以在高级页面重置附属文件,按新的设置重新处理
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandleOptions {

    /// 生成缩略图、预览图(HEIC/RAW/PSD等浏览器不能直接显示的图片)、视频首帧图
    pub thumb: bool,

    /// 生成流畅版视频:比1280大的视频缩小到1280,实况照片里的视频
    pub smooth_video: bool,

    /// 生成兼容版视频:手机原生播放不了的视频转成mp4,分辨率不变
    pub compat_video: bool,

    /// 转码使用的视频编码
    pub video_codec: VideoCodec,
}

impl Default for HandleOptions {

    /// 新建的账号默认全部开启
    fn default() -> Self {
        Self {
            thumb: true,
            smooth_video: true,
            compat_video: true,
            video_codec: VideoCodec::default(),
        }
    }
}

impl HandleOptions {

    /// 老版本接口的"是否处理上传的文件":true全部处理 false全部不处理,视频编码不变
    pub fn from_legacy(handle_file: bool, video_codec: VideoCodec) -> Self {
        Self {
            thumb: handle_file,
            smooth_video: handle_file,
            compat_video: handle_file,
            video_codec,
        }
    }

    /// 读取用户记录里的设置
    pub fn of(user: &User) -> Self {
        Self {
            thumb: user.handleThumb == 1,
            smooth_video: user.handleSmoothVideo == 1,
            compat_video: user.handleCompatVideo == 1,
            video_codec: VideoCodec::parse(&user.videoCodec).unwrap_or_default(),
        }
    }

    /// 读取账号的设置
    pub async fn load(user_id: i64) -> Result<Self, String> {
        let user = user_dao::select_one(&mut lib_db::get_context(), user_id)
            .await
            .map_err(|_| format!("用户:{}不存在", user_id))?;
        Ok(Self::of(&user))
    }

    /// 写入新建的用户记录
    pub fn apply_to(&self, user: &mut User) {
        user.handleThumb = flag(self.thumb);
        user.handleSmoothVideo = flag(self.smooth_video);
        user.handleCompatVideo = flag(self.compat_video);
        user.videoCodec = self.video_codec.name().to_string();
    }

    /// 保存账号的设置
    pub async fn save(&self, ctx: &mut DbContext, user_id: i64) -> Result<(), sqlx::Error> {
        let param = user_dao::SetHandleOptionsParam {
            id: user_id,
            handle_thumb: flag(self.thumb),
            handle_smooth_video: flag(self.smooth_video),
            handle_compat_video: flag(self.compat_video),
            video_codec: self.video_codec.name().to_string(),
        };
        user_dao::set_handle_options(ctx, param).await?;
        Ok(())
    }
}

/// 数据库里的开关 0:关闭 1:开启
fn flag(value: bool) -> i64 {
    value as i64
}
