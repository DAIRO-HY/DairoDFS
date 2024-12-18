package String

import (
	"crypto/md5"
	"encoding/hex"
	"strings"
)

// 将字符串转换成md5
func ToMd5(str string) string {
	hash := md5.Sum([]byte(str))
	return hex.EncodeToString(hash[:])
}

// 获取上级目录路径
func GetParentPath(path string) string {
	
	//统一分隔符
	tempPath := strings.ReplaceAll(path, "\\", "/")
	lastSplitChar := strings.LastIndex(tempPath, "/")
	if lastSplitChar == -1 {
		return ""
	}
	return path[:lastSplitChar]
}
