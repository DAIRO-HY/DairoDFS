package String

import (
	"DairoDFS/application"
	"crypto/md5"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"math/rand"
	"strconv"
	"strings"
	"time"
)

// 将字符串转换成md5
func ToMd5(str string) string {
	hash := md5.Sum([]byte(str))
	return hex.EncodeToString(hash[:])
}

// 将字节数组转base64字符串
func ToBase64(data []byte) string {
	return base64.StdEncoding.EncodeToString(data)
}

/**
 * 获取文件名
 */
func FileName(path string) string {
	splitIndex := strings.LastIndex(path, "/")
	if splitIndex == -1 {
		return path
	}
	return path[splitIndex+1:]
}

// 获取文件后缀名
func FileExt(path string) string {
	splitIndex := strings.LastIndex(path, ".")
	if splitIndex == -1 { //根目录文件,没有父级文件夹
		return ""
	}
	return path[splitIndex+1:]
}

// 获取上级目录路径
func FileParent(path string) string {

	//统一分隔符
	tempPath := strings.ReplaceAll(path, "\\", "/")
	lastSplitChar := strings.LastIndex(tempPath, "/")
	if lastSplitChar == -1 {
		return ""
	}
	return path[:lastSplitChar]
}

/**
 * 将数字转换成较短的字母组合
 */
func ToShortString(target int64) string {
	target -= 1
	charLen := int64(len(application.SHORT_CHAR))
	shortSB := ""
	for {
		index := target % charLen
		shortSB = application.SHORT_CHAR[index:index+1] + shortSB
		target /= charLen
		if target == 0 {
			break
		}
	}
	return shortSB
}

// 生成一段随机数字
func MakeRandNumber(count int) string {
	sourceStr := "0123456789"
	return MakeRandStrBySourceStr(sourceStr, count)
}

// 生成一段随机字符串
func MakeRandStr(count int) string {
	sourceStr := "0123456789QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm"
	return MakeRandStrBySourceStr(sourceStr, count)
}

// 生成一段随机字符串
func MakeRandStrBySourceStr(sourceStr string, count int) string {
	source := rand.NewSource(time.Now().UnixMicro())
	r := rand.New(source)
	target := ""
	for i := 0; i < count; i++ {
		index := r.Intn(len(sourceStr))
		target += sourceStr[index : index+1]
	}
	return target
}

// 将某个值转换成字符串
func ValueOf(v any) string {
	if v == nil {
		return ""
	}
	switch value := v.(type) {
	case int:
		return strconv.Itoa(value)
	case int8:
		return strconv.Itoa(int(value))
	case int16:
		return strconv.Itoa(int(value))
	case int32:
		return strconv.Itoa(int(value))
	case int64:
		return strconv.FormatInt(value, 10)
	case float64:
		return floatToStr(value)
	case float32:
		return floatToStr(float64(value))
	default:
		return fmt.Sprintf("%q", v)
	}
}

// 浮点型转字符串,去掉后面的0
func floatToStr(f float64) string {
	result := strconv.FormatFloat(f, 'f', 32, 64)
	results := strings.Split(result, ".")
	fraction, _ := strconv.ParseInt(results[1], 10, 64)
	if fraction == 0 { //没有小数部分
		return results[0]
	} else {
		return results[0] + strconv.FormatInt(fraction, 10)
	}
}
