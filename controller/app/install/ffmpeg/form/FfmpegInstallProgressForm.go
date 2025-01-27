package form

type FfmpegInstallProgressForm struct {

	/** 是否正在下载 **/
	HasRuning bool `json:"hasRuning"`

	/** 是否已经安装完成 **/
	HasFinish bool `json:"hasFinish"`

	/** 文件总大小 **/
	Total string `json:"total"`

	/** 已经下载大小 **/
	DownloadedSize string `json:"downloadedSize"`

	/** 下载速度 **/
	Speed string `json:"speed"`

	/** 下载进度 **/
	Progress int `json:"progress"`

	/** 下载url **/
	Url string `json:"url"`

	/** 安装信息 **/
	Info string `json:"info"`

	/** 错误信息 **/
	Error string `json:"error"`
}
