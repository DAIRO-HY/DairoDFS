package install

import (
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"archive/zip"
	"bytes"
	"encoding/json"
	"fmt"
	"github.com/gorilla/websocket"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"sync"
	"time"
)

// 依赖程序下载信息
type LibDownloadInfo struct {

	//下载地址
	Url string

	//保存路径
	SavePath string

	// 标记是否正在安装中
	IsRuning bool

	// 是否已经安装完成
	IsInstalled bool

	// 标记下载完成
	isDownloaded bool

	/**
	 * 下载信息
	 */
	Info string

	/**
	 * 文件总大小
	 */
	Total int64

	/**
	 * 记录最后一次请求下载大小(用来计算网速)
	 */
	LastDownloadedSize int64

	/**
	 * 记录最后一次请求进度时间
	 */
	LastProgressTime int64

	// 创建一个 bytes.Buffer 来保存数据到内存
	Data bytes.Buffer
}

var lock sync.Mutex

// 下载并解压
func (mine *LibDownloadInfo) DownloadAndUnzip(validate func() error, doInstall func()) {
	lock.Lock()
	if validate() == nil { //安装已经完成
		lock.Unlock()
		return
	}
	if mine.IsRuning { //正在下载中
		lock.Unlock()
		return
	}
	mine.IsRuning = true
	lock.Unlock()
	defer func() {
		lock.Lock()
		mine.IsRuning = false
		lock.Unlock()
	}()

	//删除之前的安装目录
	os.RemoveAll(mine.SavePath)
	mine.isDownloaded = false
	mine.Info = "准备下载"

	// 创建HTTP GET请求
	resp, err := http.Get(mine.Url)
	if err != nil {
		mine.Info = fmt.Sprintf("下载失败：%q", err)
		return
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		mine.Info = fmt.Sprintf("安装失败：HttpStatus:%d", resp.StatusCode)
		return
	}

	//得到文件总大小
	mine.Total = resp.ContentLength
	mine.Info = "下载中"

	// 将响应体写入内存
	_, err = io.Copy(&mine.Data, resp.Body)
	if err != nil {
		mine.Info = fmt.Sprintf("安装失败：%q", err)
		return
	}
	mine.Info = "正在解压"

	//解压安装包
	unzipErr := mine.unzip()

	//资源回收
	mine.Data.Reset()
	if unzipErr != nil {
		mine.Info = fmt.Sprintf("安装失败：%q", unzipErr)
		return
	}
	mine.isDownloaded = true

	//去执行安装
	doInstall()

	//去验证安装结果
	validate()
}

// unzip 解压 zip 文件到目标目录
func (mine *LibDownloadInfo) unzip() error {

	// 创建一个 zip.Reader 来读取缓冲区中的 zip 数据
	r, err := zip.NewReader(bytes.NewReader(mine.Data.Bytes()), int64(mine.Data.Len()))
	if err != nil {
		return err
	}
	unzipToTempFolder := mine.SavePath + ".temp"

	// 遍历 zip 文件中的每个文件
	for index, f := range r.File {
		mine.Info = "正在解压第" + String.ToString(index+1) + "个文件"

		// 检查路径安全，防止 Zip Slip 漏洞
		if strings.Contains(f.Name, "..") {
			return fmt.Errorf("非法文件路径: %s", f.Name)
		}
		filePath := filepath.Join(unzipToTempFolder, f.Name)
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

			//0755:将解压后的文件直接赋予可执行权限，这样就省去了解压之后再去赋予权限的步骤 @TODO:Mac系统是否可用，待验证
			outFile, openErr := os.OpenFile(filePath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0755)
			if openErr != nil {
				return openErr
			}

			// 打开 zip 文件内容
			rc, openZipErr := f.Open()
			if openZipErr != nil {
				outFile.Close()
				return openZipErr
			}

			// 将内容写入解压文件
			_, saveFileErr := io.Copy(outFile, rc)

			//销毁资源
			outFile.Close()
			rc.Close()
			if saveFileErr != nil {
				return saveFileErr
			}
		}
	}

	// 重命名文件夹
	renameErr := os.Rename(unzipToTempFolder, mine.SavePath)
	return renameErr
}

// 当前安装进度
func (mine *LibDownloadInfo) SendProgress(writer http.ResponseWriter, request *http.Request) {
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
		var progressInfo LibInstallProgressForm
		progressInfo = mine.getDownloadInfo() //获取下载信息
		jsonData, _ := json.Marshal(progressInfo)
		if slices.Equal(preJsonData, jsonData) { //比较两次发送的数据，完全一样则无需发送
			if !mine.IsRuning { //安装没有在进行或者安装已经结束
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
}

/**
 * 获取下载进度
 */
func (mine *LibDownloadInfo) getDownloadInfo() LibInstallProgressForm {
	outForm := LibInstallProgressForm{
		IsRuning:    mine.IsRuning,
		Info:        mine.Info,
		IsInstalled: mine.IsInstalled,
	}
	if mine.IsInstalled { //安装完成
		return outForm
	}
	if !mine.IsRuning { //还没有开始安装
		if mine.Info == "" {
			outForm.Info = "点击“安装”按钮开始安装。"
		}
		return outForm
	}

	var currentDownloadedSize int64
	if mine.isDownloaded { //如果下载解压已经完成
		currentDownloadedSize = mine.Total
	} else {
		currentDownloadedSize = int64(mine.Data.Len())
	}
	now := time.Now().UnixMilli()
	if now == mine.LastProgressTime { //避免发生除0错误
		return outForm
	}

	//计算下载速度
	speed := (currentDownloadedSize - mine.LastDownloadedSize) / (now - mine.LastProgressTime)
	mine.LastProgressTime = now
	mine.LastDownloadedSize = currentDownloadedSize

	total := mine.Total
	if total <= 0 {
		total = currentDownloadedSize
	}

	//        form.url = FFMPEGInstallAppController.DOWNLOAD_URL
	outForm.Total = Number.ToDataSize(total)
	outForm.DownloadedSize = Number.ToDataSize(currentDownloadedSize)
	outForm.Speed = Number.ToDataSize(speed*1000) + "/S"

	//下载百分比
	prog := 0.0
	if total > 0 {
		prog = float64(currentDownloadedSize) / float64(total)
	}
	outForm.Progress = int(prog * 100)
	return outForm
}
