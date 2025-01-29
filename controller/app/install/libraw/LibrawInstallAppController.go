package libraw

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
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
		return "https://www.libraw.org/data/LibRaw-0.21.2.zip"
	case "windows":
		return "https://www.libraw.org/data/LibRaw-0.21.2-Win64.zip"
	case "darwin":
		return "https://www.libraw.org/data/LibRaw-0.21.2-Win64.zip"
	default:
		return ""
	}
}

/**
 * 页面初始化
 */
//@Get:/install_libraw.html
//@Html:app/install/install_libraw.html
func Html() {
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.LibrawPath,
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
				downloadInfo.Info = "正在安装：" + String.ToString(installResultSize) + "/" + String.ToString(installTotalSize)
			}
		})
		if installCmdErr != nil {
			downloadInfo.Info = fmt.Sprintf("安装失败：%q", installCmdErr)
			return
		}

		//case "darwin":
		//}

		//    } catch (e: Exception) {
		//        form.hasFinish = false
		//        val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
		//        if (osName == "linux") {
		//            if (e.message!!.contains("error=13")) {
		//
		//                //开启可执行权限
		//                ShellUtil.exec(
		//                    """chmod -R +x "${
		//                        File(Constant.LIBRAW_PATH).absolutePath.replace(
		//                            "/./",
		//                            "/"
		//                        )
		//                    }""""
		//                )
		//                val version = ShellUtil.exec("${Constant.LIBRAW_PATH}/libraw -version")
		//                form.info = "安装完成:$version"
		//                form.hasFinish = true
		//                return form
		//            }
		//        } else if (osName.contains("mac")) {//mac系统是
		//            if (e.message!!.contains("error=13")) {
		//                form.error =
		//                    "安装失败:$e\n解决方案:请在弹出的Terminal窗口中验证密码,然后回到当前页面,再次点击安装按钮即可."
		//
		//                // 使用 ProcessBuilder 打开终端并执行指定的命令
		//                val pb = ProcessBuilder(
		//                    "osascript",
		//                    "-e",
		//                    "tell application \"Terminal\" to do script \"sudo chmod -R +x ${
		//                        File(Constant.LIBRAW_PATH).absolutePath.replace(
		//                            "/./",
		//                            "/"
		//                        )
		//                    }\"",
		//                    "-e", "tell application \"Terminal\" to activate"
		//                )
		//                pb.start()
		//            }
		//            return form
		//        }
		//        form.error = "安装失败:$e"
	}
}

// 验证安装结果
func validate() error {
	dcrawPath, _ := filepath.Abs(application.DcrawEmuPath + "/dcraw_emu")
	_, cmdErr := ShellUtil.ExecToOkResult(dcrawPath + " -version")
	if cmdErr != nil && strings.Contains(cmdErr.Error(), "Unknown option \"-version\".") {
		downloadInfo.Info = "安装完成"
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
