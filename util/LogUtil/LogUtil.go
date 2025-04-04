package LogUtil

import (
	"DairoDFS/application"
	"fmt"
	"os"
	"strings"
	"time"
)

// 不输出日志
const LOG_OUT_TYPE_NO = 0

// 控制台输出
const LOG_OUT_TYPE_CONSOLE = 1

// 输出到文件
const LOG_OUT_TYPE_FILE = 2

// 日志存储目录
const LOG_PATH = "./data/log"

// 初始化执行
func init() {
	_, err := os.Stat(LOG_PATH)
	if os.IsNotExist(err) { //文件不存在

		// 创建多层目录
		err := os.MkdirAll(LOG_PATH, os.ModePerm)
		if err != nil {
			fmt.Println("创建文件夹./data/log失败:", err)
			return
		}
	}
}

// 记录日志
func Info(content string) {
	if !strings.Contains(application.Args.LogLevel, "info") {
		return
	}
	write("info  " + content)
}

// 记录错误日志
func Error(content string) {
	if !strings.Contains(application.Args.LogLevel, "error") {
		return
	}
	write("error  " + content)
}

// 记录错误日志
func Error2(err error) {
	if err == nil {
		return
	}
	if !strings.Contains(application.Args.LogLevel, "error") {
		return
	}
	write(fmt.Sprintf("error %q", err))
}

// 记录错误日志
func Error3(msg string, err error) {
	if err == nil {
		return
	}
	if !strings.Contains(application.Args.LogLevel, "error") {
		return
	}
	write(fmt.Sprintf("error  %s%q", msg, err))
}

// 记录错误日志
func Debug(content string) {
	if !strings.Contains(application.Args.LogLevel, "debug") {
		return
	}
	write("debug  " + content)
}

// 记录日志
func write(content string) {
	if application.Args.LogOutType == LOG_OUT_TYPE_NO { //不输出日志
		return
	}
	line := time.Now().Format("2006-01-02 15:04:05") + "  " + content + "\n"
	if application.Args.LogOutType == LOG_OUT_TYPE_CONSOLE { //控制台输出
		fmt.Print(line)
		return
	}
	logFileName := time.Now().Format("200601") + ".log"

	file, openErr := os.OpenFile(LOG_PATH+"/"+logFileName, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if openErr != nil {
		fmt.Println(openErr)
	}
	defer file.Close()

	if _, writeErr := file.WriteString(line); writeErr != nil {
		fmt.Println(writeErr)
	}
}
