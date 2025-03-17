package ImageUtil

import (
	"fmt"
	"os"
	"testing"
)

/**
 * 生成图片缩略图
 */
func TestThumbByFile(t *testing.T) {
	thumb, err := ThumbByFile("C:\\Users\\user\\Desktop\\test\\heic-diff\\1.ios.jpeg", 100)
	if err != nil {
		fmt.Println(err)
		return
	}
	//fmt.Println(thumb)
	os.WriteFile("C:\\Users\\user\\Desktop\\test\\heic-diff\\1.ios.webp", thumb, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	info, _ := GetInfo("C:\\Users\\user\\Desktop\\test\\tt.cr3.tiff")
	fmt.Println(info)
}
