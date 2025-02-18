package AESUtil

import (
	"fmt"
	"testing"
)

func TestEncrypt(t *testing.T) {

	// 要加密的字符串
	plaintext := "12345678901234567890"

	// 加密
	encrypted := Encrypt(plaintext)
	fmt.Println("加密后的字符串:", encrypted)
}

func TestDecrypt(t *testing.T) {

	// 要加密的字符串
	plaintext := "12345678901234567890"

	// 加密
	encrypted := Encrypt(plaintext)
	fmt.Println("加密后的字符串:", encrypted)

	// 解密
	decrypted, err := Decrypt(encrypted)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println("解密后的字符串:", decrypted)
}
