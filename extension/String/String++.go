package String

import (
	application "DairoDFS/appication"
	"DairoDFS/exception"
	"crypto/md5"
	"encoding/base64"
	"encoding/hex"
	"strings"
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
	splitIndex := strings.Index(path, "/")
	if splitIndex == -1 {
		return path
	}
	return path[splitIndex+1:]
}

/**
 * 获取文件后缀名
 */
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
 * 将路径分割成列表
 */
func ToDfsFileNameList(name string) ([]string, error) {
	//TODO:这里是否需要检查路径的正确行，待验证
	//DfsFileUtil.CheckPath(name)
	if len(name) == 0 {
		return []string{}, nil
	}
	if !strings.HasPrefix(name, "/") {
		return nil, exception.Biz("文件路径必须以/开头")
	}
	if strings.HasSuffix(name, "/") {
		name = name[:len(name)-1]
	}
	return strings.Split(name, "/"), nil
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
