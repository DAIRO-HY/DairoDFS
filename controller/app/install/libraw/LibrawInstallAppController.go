package libraw

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
//@Group:/app/install/libraw

// 下载信息
var downloadInfo *install.LibDownloadInfo

//go:embed libraw-install.sh
var librawInstallSH embed.FS

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-source.zip"
	case "windows":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-Win64.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-macOS.zip"
	default:
		return ""
	}
}

// @Get:
// @Html:app/install/libraw.html
func Html() {

	//清除上一步的缓存
	ffprobe.Recycle()
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.LibrawPath,
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
	case "linux":

		// 读取嵌入的文件内容
		data, _ := librawInstallSH.ReadFile("libraw-install.sh")
		targetFile := application.LibrawPath + "/libraw-install.sh"

		//0755：文件所有者可读、写、执行，同组用户和其他用户可读、执行。
		writeShFileErr := os.WriteFile(targetFile, data, 0755)
		if writeShFileErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", writeShFileErr)
			return
		}

		cache := make([]byte, 128)

		abs, _ := filepath.Abs(application.LibrawPath + "/libraw-install.sh")
		installResultSize := 0
		const installTotalSize = 250820
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
		cmd = "dcraw_emu"
	} else {
		cmd = "\"" + application.LIBRAW_BIN + "/dcraw_emu\""
	}
	_, cmdErr := ShellUtil.ExecToOkResult(cmd + " -version")
	if cmdErr != nil && strings.Contains(cmdErr.Error(), `Unknown option \"-version\".`) {
		downloadInfo.Info = "安装完成"
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
