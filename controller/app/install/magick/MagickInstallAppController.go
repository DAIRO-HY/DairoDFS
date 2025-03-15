package magick

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/controller/app/install/ffprobe"
	"DairoDFS/extension/String"
	"DairoDFS/util/ShellUtil"
	"embed"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

/**
 * 安装libraw
 */
//@Group:/app/install/magick

// 下载信息
var downloadInfo *install.LibDownloadInfo

//go:embed imagemagick-install.sh
var imagemagickInstallSH embed.FS

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return ""
	case "windows":
		return ""
	case "darwin":
		return ""
	default:
		return ""
	}
}

// @Get:
// @Html:app/install/magick.html
func Html() {

	//清除上一步的缓存
	ffprobe.Recycle()
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.ImageMagickPath,
		}
	}
	if validate() != nil {
		downloadInfo.Info = ""
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
	go downloadInfo.DownloadAndUnzip(validate, doInstall)
}

// 当前安装进度
// @Request:/progress
func Progress(writer http.ResponseWriter, request *http.Request) {
	downloadInfo.SendProgress(writer, request)
}

// 文件已经下载完成，获取安装信息
func doInstall() {
	switch runtime.GOOS {
	case "darwin":
		cache := make([]byte, 128)
		installResultSize := 0
		const installTotalSize = 210820
		_, installCmdErr := ShellUtil.ExecToOkReader("brew install imagemagick", func(rc io.ReadCloser) {
			for {
				n, err := rc.Read(cache)
				if err != nil {
					if err == io.EOF || n == 0 {
						break
					}
				}
				installResultSize += n
				downloadInfo.Info = "正在安装：" + String.ValueOf(installResultSize) + "/" + String.ValueOf(installTotalSize)
			}
		})
		if installCmdErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", installCmdErr)
			return
		}
	case "linux":

		// 读取嵌入的文件内容
		data, _ := imagemagickInstallSH.ReadFile("imagemagick-install.sh")
		targetFile := application.ImageMagickPath + "/imagemagick-install.sh"

		//0755：文件所有者可读、写、执行，同组用户和其他用户可读、执行。
		writeShFileErr := os.WriteFile(targetFile, data, 0755)
		if writeShFileErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", writeShFileErr)
			return
		}
		cache := make([]byte, 128)
		abs, _ := filepath.Abs(application.ImageMagickPath + "/imagemagick-install.sh")
		installResultSize := 0
		const installTotalSize = 210820
		_, installCmdErr := ShellUtil.ExecToOkReader(abs, func(rc io.ReadCloser) {
			for {
				n, err := rc.Read(cache)
				if err != nil {
					if err == io.EOF || n == 0 {
						break
					}
				}
				installResultSize += n
				downloadInfo.Info = "正在安装：" + String.ValueOf(installResultSize) + "/" + String.ValueOf(installTotalSize)
			}
		})
		if installCmdErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", installCmdErr)
			return
		}
	}
}

// 验证安装结果
func validate() error {
	result, cmdErr := ShellUtil.ExecToOkResult("magick --version")
	if cmdErr == nil && strings.HasPrefix(result, "Version: ImageMagick") {
		downloadInfo.Info = "安装完成"
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
