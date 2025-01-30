package VideoUtil

/**
 * 视频信息
 */
type VedioInfo struct {

	//宽
	Width int `json:"width,omitempty"`

	//高
	Height int `json:"height,omitempty"`

	//帧数
	Fps float32 `json:"fps,omitempty"`

	//视频比特率
	Bitrate int `json:"bitrate,omitempty"`

	//视频时长（毫秒）
	Duration int64 `json:"duration,omitempty"`

	//视频创建时间戳
	Date int64 `json:"date,omitempty"`

	//音频比特率
	AudioBitrate int `json:"audioBitrate,omitempty"`

	//音频采样率（HZ）
	AudioSampleRate int `json:"audioSampleRate,omitempty"`

	//音频格式
	AudioFormat string `json:"audioFormat,omitempty"`
}
