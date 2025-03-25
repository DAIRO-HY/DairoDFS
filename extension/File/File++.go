package File

import (
	"DairoDFS/exception"
	"crypto/md5"
	"encoding/hex"
	"io"
	"os"
	"regexp"
	"strings"
)

// 获取文件md5
func ToMd5(path string) string {

	// 打开文件
	file, err := os.Open(path)
	if err != nil {
		return ""
	}
	defer file.Close()
	return ToMd5ByReader(file)
}

// 将字节数组转换成md5
func ToMd5ByBytes(data []byte) string {
	hash := md5.Sum(data)
	return hex.EncodeToString(hash[:])
}

// 获取文件md5
func ToMd5ByReader(reader io.Reader) string {

	// 创建 MD5 哈希器
	hash := md5.New()

	// 将文件内容写入哈希器
	if _, err := io.Copy(hash, reader); err != nil {
		return ""
	}

	// 获取最终的哈希值
	hashSum := hash.Sum(nil)

	// 转换为十六进制字符串
	return hex.EncodeToString(hashSum)
}

// 检查文件路径是否合法
// path 文件路径
func CheckPath(path string) {
	pattern := `[>,?,\\,:,|,<,*,"]`
	matched, _ := regexp.MatchString(pattern, path)
	if matched {
		panic(exception.Biz("文件路径不能包含>,?,\\,:,|,<,*,\"字符"))
	}
	if strings.Contains(path, "//") {
		panic(exception.Biz("文件路径不能包含两个连续的字符/"))
	}
}

// 将路径分割成列表
// filePath 文件或文件夹路径
// return 拆分后的文件名数组
func ToSubNames(filePath string) []string {
	CheckPath(filePath)
	if len(filePath) == 0 {
		return []string{""}
	}
	if !strings.HasPrefix(filePath, "/") {
		panic(exception.Biz("文件路径必须以/开头"))
	}
	if strings.HasSuffix(filePath, "/") {
		filePath = filePath[:len(filePath)-1]
	}
	return strings.Split(filePath, "/")
}
