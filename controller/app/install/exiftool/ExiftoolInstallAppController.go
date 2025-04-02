package exiftool

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/controller/app/install/magick"
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
 * 安装exiftool
 */
//@Group:/app/install/exiftool

// 下载信息
var downloadInfo *install.LibDownloadInfo

//go:embed exiftool-install.sh
var exiftoolInstallSH embed.FS

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return ""
	case "windows":
		return "https://exiftool.org/exiftool-13.26_64.zip"
	case "darwin":
		return ""
	default:
		return ""
	}
}

// @Get:
// @Html:app/install/exiftool.html
func Html() {

	//清除上一步的缓存
	magick.Recycle()
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.ExiftoolPath,
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
	case "windows":

		//将exiftool(-k).exe重命名为exiftool.exe
		os.Rename(application.ExiftoolPath+"/exiftool-13.26_64/exiftool(-k).exe", application.ExiftoolPath+"/exiftool-13.26_64/exiftool.exe")
	case "darwin": //@TODO:
		cache := make([]byte, 128)
		installResultSize := 0
		const installTotalSize = 302021
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
		data, _ := exiftoolInstallSH.ReadFile("exiftool-install.sh")
		targetFile := application.ExiftoolPath + "/exiftool-install.sh"

		//先创建文件夹
		os.MkdirAll(application.ExiftoolPath, os.ModePerm)

		//0755：文件所有者可读、写、执行，同组用户和其他用户可读、执行。
		writeShFileErr := os.WriteFile(targetFile, data, 0755)
		if writeShFileErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", writeShFileErr)
			return
		}
		cache := make([]byte, 128)
		abs, _ := filepath.Abs(application.ExiftoolPath + "/exiftool-install.sh")
		installResultSize := 0
		const installTotalSize = 97943
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
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "exiftool"
	} else {
		cmd = "\"" + application.ExiftoolPath + "/exiftool-13.26_64/exiftool\""
	}
	result, cmdErr := ShellUtil.ExecToOkResult(cmd + " -ver")
	if cmdErr == nil && strings.HasPrefix(result, "13.26") {
		downloadInfo.Info = "安装完成"
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
