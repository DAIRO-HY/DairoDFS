package File

import (
	"crypto/md5"
	"encoding/hex"
	"io"
	"os"
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
