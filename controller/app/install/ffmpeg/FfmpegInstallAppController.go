package ffmpeg

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install/ffmpeg/form"
	"DairoDFS/extension/Number"
	"DairoDFS/util/ShellUtil"
	"archive/zip"
	"bytes"
	"encoding/json"
	"fmt"
	"github.com/gorilla/websocket"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"strings"
	"time"
)

/**
 * 安装ffmpeg
 */
//@Group:/app/install/ffmpeg

// 下载信息
var downloadInfo *DownloadInfo

// 标记是否正在安装中
var isRuning bool

/**
 * 页面初始化
 */
//@Get:/install_ffmpeg.html
//@Html:app/install/install_ffmpeg.html
func Html() {}

/**
 * 开始安装
 */
//@Post:/install
func Install() {
	_, err := os.Stat(application.FfmpegPath)
	if !os.IsNotExist(err) { //文件存在
		return
	}
	if isRuning { //正在下载中
		return
	}
	go downloadAndUnzip()
}

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-linux-amd64.zip"
	case "windows":
		//return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-win.zip"
		return "https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-7.0.2-essentials_build.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-macOS.zip"
	default:
		return ""
	}
}

// 下载并解压
func downloadAndUnzip() {
	defer func() {
		isRuning = false
	}()
	isRuning = true
	downloadInfo = &DownloadInfo{
		info: "准备下载",
	}

	// 创建HTTP GET请求
	resp, err := http.Get(url())
	if err != nil {
		downloadInfo.info = fmt.Sprintf("安装失败：%q", err)
		return
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		downloadInfo.info = fmt.Sprintf("安装失败：HttpStatus:%d", resp.StatusCode)
		return
	}

	//得到文件总大小
	downloadInfo.total = resp.ContentLength
	downloadInfo.info = "下载中"

	// 将响应体写入内存
	_, err = io.Copy(&downloadInfo.downloadBuffer, resp.Body)
	if err != nil {
		downloadInfo.info = fmt.Sprintf("安装失败：%q", err)
		return
	}
	downloadInfo.info = "正在解压"

	//解压安装包
	unzipErr := unzip()
	if unzipErr != nil {
		downloadInfo.info = fmt.Sprintf("安装失败：%q", unzipErr)
		return
	}
	downloadInfo.info = "安装完成"
}

// unzip 解压 zip 文件到目标目录
func unzip() error {

	// 创建一个 zip.Reader 来读取缓冲区中的 zip 数据
	r, err := zip.NewReader(bytes.NewReader(downloadInfo.downloadBuffer.Bytes()), int64(downloadInfo.downloadBuffer.Len()))
	if err != nil {
		return err
	}

	// 遍历 zip 文件中的每个文件
	for _, f := range r.File {
		filePath := filepath.Join(application.FfmpegPath, f.Name)

		// 检查路径安全，防止 Zip Slip 漏洞
		if !filepath.HasPrefix(filePath, filepath.Clean(application.FfmpegPath)+string(os.PathSeparator)) {
			return fmt.Errorf("非法文件路径: %s", filePath)
		}

		if f.FileInfo().IsDir() {
			// 创建目录
			if err = os.MkdirAll(filePath, os.ModePerm); err != nil {
				return err
			}
		} else {
			// 创建解压文件
			if err = os.MkdirAll(filepath.Dir(filePath), os.ModePerm); err != nil {
				return err
			}

			outFile, openErr := os.OpenFile(filePath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.Mode())
			if openErr != nil {
				return openErr
			}
			defer outFile.Close()

			// 打开 zip 文件内容
			rc, err := f.Open()
			if err != nil {
				return err
			}
			defer rc.Close()

			// 将内容写入解压文件
			_, err = io.Copy(outFile, rc)
			if err != nil {
				return err
			}
		}
	}
	return nil
}

// 当前安装进度
// @Request:/progress
func Progress(writer http.ResponseWriter, request *http.Request) {
	// 创建WebSocket升级器
	var upgrader = websocket.Upgrader{
		// 允许跨域请求
		CheckOrigin: func(r *http.Request) bool {
			return true
		},
	}

	// 将HTTP连接升级为WebSocket连接
	conn, err := upgrader.Upgrade(writer, request, nil)
	if err != nil {
		fmt.Println("升级为WebSocket失败:", err)
		return
	}
	defer conn.Close()

	//记录上次发送的数据，如果前后两次发送的数据一样，则不要发送数据
	var preJsonData []byte
	for {
		time.Sleep(250 * time.Millisecond)
		var progressInfo form.FfmpegInstallProgressForm
		_, existsErr := os.Stat(application.FfmpegPath)
		if os.IsNotExist(existsErr) { //文件未下载
			progressInfo = getDownloadInfo() //获取下载信息
		} else {
			progressInfo = getInstallInfo() //获取安装信息
		}
		jsonData, _ := json.Marshal(progressInfo)
		if slices.Equal(preJsonData, jsonData) { //比较两次发送的数据，完全一样则无需发送
			if !isRuning { //安装没有在进行或者安装已经结束
				break
			}
			continue
		}

		// 发送消息
		err = conn.WriteMessage(websocket.TextMessage, jsonData)
		if err != nil {
			break
		}
		preJsonData = jsonData
	}
	downloadInfo = nil
}

/**
 * 获取下载进度
 */
func getDownloadInfo() form.FfmpegInstallProgressForm {
	outForm := form.FfmpegInstallProgressForm{
		IsRuning: isRuning,
	}
	if downloadInfo == nil { //还没有开始安装
		outForm.Info = "没有安装，点击“安装”按钮开始安装FFMPEG。"
		return outForm
	}
	outForm.Info = downloadInfo.info
	currentDownloadedSize := int64(downloadInfo.downloadBuffer.Len())
	now := time.Now().UnixMilli()

	//计算下载速度
	speed := (currentDownloadedSize - downloadInfo.lastDownloadedSize) / (now - downloadInfo.lastProgressTime)

	downloadInfo.lastProgressTime = now
	downloadInfo.lastDownloadedSize = currentDownloadedSize

	//        form.url = FFMPEGInstallAppController.DOWNLOAD_URL
	outForm.Total = Number.ToDataSize(downloadInfo.total)
	outForm.DownloadedSize = Number.ToDataSize(currentDownloadedSize)
	outForm.Speed = Number.ToDataSize(speed*1000) + "/S"

	//下载百分比
	prog := 0.0
	if downloadInfo.total > 0 {
		prog = float64(currentDownloadedSize) / float64(downloadInfo.total)
	}
	outForm.Progress = int(prog * 100)
	return outForm
}

// 文件已经下载完成，获取安装信息
func getInstallInfo() form.FfmpegInstallProgressForm {
	outForm := form.FfmpegInstallProgressForm{}
	versionResult, cmdErr := ShellUtil.ExecToOkResult(application.FfmpegPath + "/ffmpeg -version")
	if cmdErr == nil {
		outForm.Info = "安装完成：" + versionResult
		outForm.IsInstalled = true
		return outForm
	}
	switch runtime.GOOS {
	case "windows":
		outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
	case "linux":
		if strings.Contains(cmdErr.Error(), "error=13") { //没有赋予可执行权限

			//开启可执行权限
			_, versionCmdErr := ShellUtil.ExecToOkResult("chmod -R +x " + application.FfmpegPath)
			if versionCmdErr != nil {
				outForm.Info = fmt.Sprintf("安装失败：%q", versionCmdErr)
				return outForm
			}

			//再次获取版本号
			versionResult, cmdErr = ShellUtil.ExecToOkResult(application.FfmpegPath + "/ffmpeg -version")
			if cmdErr == nil {
				outForm.Info = "安装完成：" + versionResult
				outForm.IsInstalled = true
				return outForm
			}
			outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
		}
	case "darwin":
	}

	//    } catch (e: Exception) {
	//        form.hasFinish = false
	//        val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
	//        if (osName == "linux") {
	//            if (e.message!!.contains("error=13")) {
	//
	//                //开启可执行权限
	//                ShellUtil.exec(
	//                    """chmod -R +x "${
	//                        File(Constant.FFMPEG_PATH).absolutePath.replace(
	//                            "/./",
	//                            "/"
	//                        )
	//                    }""""
	//                )
	//                val version = ShellUtil.exec("${Constant.FFMPEG_PATH}/ffmpeg -version")
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
	//                        File(Constant.FFMPEG_PATH).absolutePath.replace(
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
	//    }
	return outForm
}
