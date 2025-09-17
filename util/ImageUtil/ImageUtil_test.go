package ImageUtil

import (
	"DairoDFS/application"
	"fmt"
	"os"
	"testing"
	"time"
)

func init() {
	application.FfmpegPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffmpeg"
}

func TestGetInfo(t *testing.T) {
	info1, _ := GetInfo("C:\\test\\1.cr3.jpg")
	fmt.Println(info1)

	//info2, _ := GetInfo("C:\\test\\2.jpg")
	//fmt.Println(info2)
}

func TestToJpg(t *testing.T) {
	jpgData, _ := ToJpg("C:\\test\\mov\\1.jpg", 100)
	os.WriteFile("C:\\test\\mov\\1.jpg.jpg", jpgData, 0644)
}

// TestThumbByData - 生成图片缩略图
func TestResizeByData(t *testing.T) {
	data, _ := os.ReadFile("C:\\test\\s.jpg")
	var thumb []byte
	var err error

	now := time.Now()
	for i := 0; i < 10; i++ {
		thumb, err = ResizeByData(data, 800, 85)
	}
	fmt.Println(time.Now().Sub(now).Seconds())
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("C:\\test\\s.resize.jpg", thumb, os.ModePerm)
}

// TestThumbSizeByData - 生成图片缩略图
func TestCropByData(t *testing.T) {
	data, _ := os.ReadFile("C:\\test\\1.cr3.jpg")
	var thumb []byte
	var err error

	now := time.Now()
	for i := 0; i < 10; i++ {
		thumb, err = CropByData(data, 200, 200, 85)
	}
	fmt.Println(time.Now().Sub(now).Seconds())
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("C:\\test\\1.crop.jpg", thumb, os.ModePerm)
}
