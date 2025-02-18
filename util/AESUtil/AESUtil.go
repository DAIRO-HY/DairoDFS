package AESUtil

import (
	"DairoDFS/application"
	"DairoDFS/extension/String"
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"io"
	"os"
)

// AES密钥
var key []byte

func init() {
	data, err := os.ReadFile(application.DataPath + "/AES_KEY")
	if err == nil {
		key = data
		return
	}

	//生成一个随机16位密钥,密钥必须是 16, 24 或 32 字节长度的 AES 密钥
	key = []byte(String.MakeRandStr(16))

	//将密钥写入文件，下次直接使用
	os.WriteFile(application.DataPath+"/AES_KEY", key, 0644)
}

// 加密函数
func Encrypt(plaintext string) string {
	block, _ := aes.NewCipher(key)

	// 将明文转换为字节切片
	plaintextBytes := []byte(plaintext)

	// 创建一个与明文长度相同的密文切片，并附加一个额外的块大小
	ciphertext := make([]byte, aes.BlockSize+len(plaintextBytes))

	// 生成一个随机的初始化向量 (IV)
	iv := ciphertext[:aes.BlockSize]
	//if _, err := io.ReadFull(rand.Reader, iv); err != nil {
	//	return "", err
	//}
	io.ReadFull(rand.Reader, iv)

	// 使用 CFB 模式加密
	stream := cipher.NewCFBEncrypter(block, iv)
	stream.XORKeyStream(ciphertext[aes.BlockSize:], plaintextBytes)

	// 返回 base64 编码的密文
	return base64.URLEncoding.EncodeToString(ciphertext)
}

// 解密函数
func Decrypt(encrypted string) (string, error) {
	block, _ := aes.NewCipher(key)

	// 解码 base64 编码的密文
	ciphertext, err := base64.URLEncoding.DecodeString(encrypted)
	if err != nil {
		return "", err
	}

	// 检查密文长度是否合法
	if len(ciphertext) < aes.BlockSize {
		return "", errors.New("密文长度不合法")
	}

	// 提取初始化向量 (IV)
	iv := ciphertext[:aes.BlockSize]
	ciphertext = ciphertext[aes.BlockSize:]

	// 使用 CFB 模式解密
	stream := cipher.NewCFBDecrypter(block, iv)
	stream.XORKeyStream(ciphertext, ciphertext)

	// 返回解密后的字符串
	return string(ciphertext), nil
}
