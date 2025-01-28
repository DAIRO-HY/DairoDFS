package form

type FfprobeInstallProgressForm struct {

	/** 是否正在下载 **/
	IsRuning bool `json:"isRuning"`

	/** 是否已经安装完成 **/
	IsInstalled bool `json:"isInstalled,omitempty"`

	/** 文件总大小 **/
	Total string `json:"total"`

	/** 已经下载大小 **/
	DownloadedSize string `json:"downloadedSize"`

	/** 下载速度 **/
	Speed string `json:"speed"`

	/** 下载进度 **/
	Progress int `json:"progress"`

	/** 安装信息 **/
	Info string `json:"info"`
}
