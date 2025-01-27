package application

import (
	"DairoDFS/util/LogUtil"
	"fmt"
	"os"
	"strconv"
	"strings"
)

/**
 * 基准时间戳
 * 用当前时间戳减去此时间戳,得到唯一字符串,一旦数据库已经有数据,该值就不能改,否则可能导致重数据.
 */
const BASE_TIME = 1699177026571

// 回收站保留天数
const TRASH_TIMEOUT = 30

/** 用户登录token的cookie键 **/
const COOKIE_TOKEN = "token"

/** 用户ID键 **/
const USER_ID = "USER_ID"

/**
 * 数字转换成短文本支持的字符串
 */
const SHORT_CHAR = "0Mkhc7EingwxJYtPdUmWGHeV3ND5KRACb4rBXlO6f91syvIuqoZQLa2FTS8zpj"

/**
 * 账户信息存储路径
 */
const SYSTEM_JSON_PATH = "./data/system.json"

/**
 * 登录用户ID
 */
const REQUEST_USER_ID = "USER_ID"

/**
 * 是否管理员
 */
const REQUEST_IS_ADMIN = "IS_ADMIN"

/**
 * DB文件路径
 */
var DbPath = "./data/dairo-dfs.sqlite"

/**
 * 数据存放文件夹
 */
var DataPath = "./data"

/**
 * ffmpeg安装目录
 */
var FfmpegPath = DataPath + "/ffmpeg"

/**
 * ffprobe安装目录
 */
var FfprobePath string

/**
 * libraw安装目录
 */
var LibrawPath string

// 是否开发模式
var IsDev bool

// WEB管理端口
var WebPort = 8031

// 用一个用户允许登录的客户端数量限制
var UserTokenLimit = 10

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
	//for index, it := range os.Args {
	//	switch it {
	//	case "-web-port":
	//		WebPort, _ = strconv.Atoi(paramArr[1])
	//	case "-log-type": //日志输出方式
	//		switch paramArr[1] {
	//		case "0":
	//			LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_NO
	//		case "1":
	//			LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_CONSOLE
	//		case "2":
	//			LogUtil.LogOutType = LogUtil.LOG_OUT_TYPE_FILE
	//		}
	//	case "-log-level": //日志输出级别
	//		levels := strings.Split(paramArr[1], ",")
	//		for _, level := range levels {
	//			LogUtil.LogLevel[level] = true
	//		}
	//	case "-is-dev":
	//		IsDev = paramArr[1] == "true"
	//	}
	//}
}

// 防止程序终止
func StopRuntimeError() {
	if r := recover(); r != nil {
		//防止程序终止
	}
}
