package ffmpeg

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/util/ShellUtil"
	"net/http"
	"runtime"
)

/**
 * 安装ffmpeg
 */
//@Group:/app/install/ffmpeg

// 下载信息
var downloadInfo *install.LibDownloadInfo

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-linux-amd64.zip"
	case "windows":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-win.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-macOS.zip"
	default:
		return ""
	}
}

/**
 * 页面初始化
 */
//@Get:/install_ffmpeg.html
//@Html:app/install/install_ffmpeg.html
func Html() {
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.FfmpegPath,
		}
	}
	if validate() != nil {
		downloadInfo.IsInstalled = false
	}
}

// 资源回收
// @Post:/recycle
func Recycle() {
	downloadInfo = nil
}

/**
 * 开始安装
 */
//@Post:/install
func Install() {
	go downloadInfo.DownloadAndUnzip(validate, nil)
}

// 当前安装进度
// @Request:/progress
func Progress(writer http.ResponseWriter, request *http.Request) {
	downloadInfo.SendProgress(writer, request)
}

// 验证安装结果
func validate() error {
	versionResult, cmdErr := ShellUtil.ExecToOkResult(application.FfmpegPath + "/ffmpeg -version")
	if cmdErr == nil {
		downloadInfo.Info = "安装完成：" + versionResult
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
