package appication

import (
	"DairoDFS/util/LogUtil"
	"fmt"
	"os"
	"strconv"
	"strings"
)

// 是否开发模式
var IsDev bool

// WEB管理端口
var WebPort = 9030

func Init() {
	fmt.Println("------------------------------------------------------------------------")
	for _, it := range os.Args {
		fmt.Println(it)
	}
	fmt.Println("------------------------------------------------------------------------")
	for _, it := range os.Args {
		paramArr := strings.Split(it, ":")
		switch paramArr[0] {
		case "-web-port":
			WebPort, _ = strconv.Atoi(paramArr[1])
		case "-log-type": //日志输出方式
			switch paramArr[1] {
			case "0":
				LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_NO
			case "1":
				LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_CONSOLE
			case "2":
				LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_FILE
			}
		case "-log-level": //日志输出级别
			levels := strings.Split(paramArr[1], ",")
			for _, level := range levels {
				LogUtil.LogLevel[level] = true
			}
		case "-is-dev":
			IsDev = paramArr[1] == "true"
		}
	}
}
