package advanced

import (
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/File"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBUtil"
	"net/http"
	"os"
	"sync"
	"time"
)

// 高级功能：文件验证
//@Group:/app/advanced

// 文件验证同步锁
var validateFileMd5Lock sync.Mutex

// 当前正在文件验证的writer
var validateFileMd5Writer *http.ResponseWriter

var mPreLockCond *sync.Cond

// 停止标记
var isStopFlag bool

// 标记是否正在运行
var isRunning bool

// 当前正在验证的文件数量
var storageLen int

// 验证失败的文件
var validateErrFile []string

// 验证文件完整
// @Request:/validate_file_md5
func ValidateFileMD5(writer http.ResponseWriter, isInit bool) {
	var preLockCond *sync.Cond
	validateFileMd5Lock.Lock()
	{
		if isInit && !isRunning { //画面初始化时，如果验证程序没有开始，不做任何处理
			validateFileMd5Lock.Unlock()
			return
		}
		if !isInit && isRunning { //非画面初始化时，如果验证程序正在进行，则终止验证程序
			isStopFlag = true
			mPreLockCond.Broadcast()
			validateFileMd5Lock.Unlock()
			return
		}

		//设置头部信息，保持连接
		writer.Header().Set("Content-Type", "text/event-stream")
		writer.Header().Set("Cache-Control", "np-cache")
		writer.Header().Set("Connection", "keep-alive")

		//更新写入流
		validateFileMd5Writer = &writer
		if isRunning { //如果程序正在运行

			//发送总数
			writer.Write([]byte("event:total\ndata:" + String.ValueOf(storageLen) + "\n\n"))
			writer.(http.Flusher).Flush()
		} else {
			isStopFlag = false
			isRunning = true
			validateErrFile = nil
			go openValidateFileMD5()
		}
		if mPreLockCond != nil {

			//通知结束掉上次的验证等待
			mPreLockCond.Broadcast()
		}
		preLockCond = sync.NewCond(new(sync.Mutex))
		mPreLockCond = preLockCond
	}
	validateFileMd5Lock.Unlock()
	preLockCond.L.Lock()
	preLockCond.Wait()
	preLockCond.L.Unlock()
}

// 验证文件完整
func openValidateFileMD5() {
	storageList := DBUtil.SelectList[dto.StorageFileDto]("select md5,path from main.storage_file")
	storageLen = len(storageList)
	validateFileMd5Lock.Lock()
	{
		//发送总数
		(*validateFileMd5Writer).Write([]byte("event:total\ndata:" + String.ValueOf(storageLen) + "\n\n"))
	}
	validateFileMd5Lock.Unlock()
	for i, it := range storageList {
		if _, err := os.Stat(it.Path); os.IsNotExist(err) {
			validateErrFile = append(validateErrFile, it.Md5+"：找不到文件路径："+it.Path+"<br>")
		} else {
			md5 := File.ToMd5(it.Path)
			if md5 != it.Md5 {
				validateErrFile = append(validateErrFile, it.Md5+"：文件不完整，当前文件MD5："+md5+"<br>")
			}
		}
		validateFileMd5Lock.Lock()
		{
			if isStopFlag {
				validateFileMd5Lock.Unlock()
				break
			}
			if _, err := (*validateFileMd5Writer).Write([]byte("data:" + String.ValueOf(i+1) + "\n\n")); err != nil { //可能客户端已经被关闭
				validateFileMd5Lock.Unlock()
				break
			}
			(*validateFileMd5Writer).(http.Flusher).Flush()
		}
		validateFileMd5Lock.Unlock()

		//测试用
		time.Sleep(100 * time.Millisecond)
	}

	//资源回收
	validateFileMd5Lock.Lock()
	if !isStopFlag { //发送结束事件
		if len(validateErrFile) == 0 {
			validateErrFile = append(validateErrFile, "文件数据完成，检测时间："+Date.FormatDate(time.Now()))
		}
		(*validateFileMd5Writer).Write([]byte("event:finish\ndata:\n\n"))
		(*validateFileMd5Writer).(http.Flusher).Flush()
	}
	validateFileMd5Writer = nil

	//通知结束掉上次的验证等待
	mPreLockCond.Broadcast()
	mPreLockCond = nil
	isRunning = false

	validateFileMd5Lock.Unlock()
}
