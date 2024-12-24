package VideoUtil

/**
 * 视频信息
 */
type VedioInfo struct {

	//宽
	Width int

	//高
	Height int

	//帧数
	Fps float32

	//视频比特率
	Bitrate int

	//视频时长（毫秒）
	Duration int64

	//视频创建时间戳
	Date int64

	//音频比特率
	AudioBitrate int

	//音频采样率（HZ）
	AudioSampleRate int

	//音频格式
	AudioFormat string
}
