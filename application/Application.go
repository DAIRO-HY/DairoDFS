package application

import (
	"fmt"
	"log"
	"os"
	"reflect"
	"strconv"
	"strings"
	"unicode"
)

// VERSION 版本号
const VERSION = "1.0.19-RC"

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

// 数据存放文件夹
var DataPath = "./data"

// 数据库存放文件夹
var SQLITE_PATH = DataPath + "/dairo-dfs.sqlite"

// 数据存放文件夹
var TEMP_PATH = DataPath + "/temp"

// ffmpeg安装目录
var FfmpegPath = DataPath + "/ffmpeg"

// ffprobe安装目录
var FfprobePath = DataPath + "/ffprobe"

// libraw安装目录
var LibrawPath = DataPath + "/libraw"

// imagemagick安装目录
var ImageMagickPath = DataPath + "/imagemagick"

// Exiftool安装目录
var ExiftoolPath = DataPath + "/exiftool"

// libraw中的Dcraw模拟器存放路径
var LIBRAW_BIN = LibrawPath + "/LibRaw-0.21.2/bin"

// 用一个用户允许登录的客户端数量限制
var UserTokenLimit = 10

// 程序启动参数
var Args appArgs

// 程序启动参数
type appArgs struct {

	// WEB端口
	Port int

	// 是否开发模式
	IsDev bool
}

func Init() {
	parseArgs()

	//创建临时目录
	os.MkdirAll(TEMP_PATH, os.ModePerm)
}

// 解析参数
func parseArgs() {
	fmt.Println("------------------------------------------------------------------------")
	fmt.Println(strings.Join(os.Args, " "))
	fmt.Println("------------------------------------------------------------------------")
	Args = appArgs{
		Port:  8031,
		IsDev: false,
	}
	argsElem := reflect.ValueOf(&Args).Elem()

	for i := 0; i < len(os.Args); i++ {
		key := os.Args[i]
		if !strings.HasPrefix(key, "--") {
			continue
		}

		filedName := ""
		for _, it := range strings.Split(key[2:], "-") {
			if len(it) == 0 {
				continue
			}
			// 将字符串转换为 rune 切片以便处理 Unicode 字符
			r := []rune(it)

			// 将首字母大写
			r[0] = unicode.ToUpper(r[0])
			filedName += string(r)
		}
		field := argsElem.FieldByName(filedName)
		if !field.IsValid() { //如果该字段不存在
			continue
		}
		if i+1 > len(os.Args)-1 { //已经没有下一个元素
			break
		}

		//参数值
		value := os.Args[i+1]
		switch field.Kind() {

		//整数类型转换
		case reflect.Int,
			reflect.Int8,
			reflect.Int16,
			reflect.Int32,
			reflect.Int64,
			reflect.Uint,
			reflect.Uint8,
			reflect.Uint16,
			reflect.Uint32,
			reflect.Uint64:
			v, err := strconv.ParseInt(value, 10, 64)
			if err != nil {
				log.Panicf("参数%s=%s发生了转换错误:%q", key, value, err)
			}
			field.SetInt(v)
		case reflect.Float32, reflect.Float64:
			v, err := strconv.ParseFloat(value, 64)
			if err != nil {
				log.Panicf("参数%s=%s发生了转换错误:%q", key, value, err)
			}
			field.SetFloat(v)
		case reflect.Bool:
			v, err := strconv.ParseBool(value)
			if err != nil {
				log.Panicf("参数%s=%s发生了转换错误:%q", key, value, err)
			}
			field.SetBool(v)
		case reflect.String:
			field.SetString(value)
		}
		i++
	}
	//for _, it := range os.Args {
	//	paramArr := strings.Split(it, ":")
	//	switch paramArr[0] {
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
