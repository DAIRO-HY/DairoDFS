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

/**
 * 生成图片缩略图
 */
func TestThumbByFile(t *testing.T) {
	thumb, err := ThumbByFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3", 100, 85)
	if err != nil {
		fmt.Println(err)
		return
	}
	//fmt.Println(thumb)
	os.WriteFile("./data/bb.jpeg", thumb, os.ModePerm)
}

func TestPng2Jpg(t *testing.T) {
	data, _ := os.ReadFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\heic\\hh.png")
	jpgData, _ := Png2Jpg(data, 2)

	os.WriteFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\heic\\hh.png.2.jpg", jpgData, 0644)
}

func TestGetInfo(t *testing.T) {
	info, _ := GetInfo("C:\\Users\\user\\Desktop\\test\\tt.cr3.tiff")
	fmt.Println(info)
}

// TestThumbByData - 生成图片缩略图
func TestThumbByData(t *testing.T) {
	data, _ := os.ReadFile("C:\\test\\big.jpg")
	var thumb []byte
	var err error

	now := time.Now()
	for i := 0; i < 1; i++ {
		thumb, err = ThumbByData(data, 1300, 85)
	}
	fmt.Println(time.Now().Sub(now).Seconds())
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("C:\\test\\out-go.jpg", thumb, os.ModePerm)
}

// TestThumbSizeByData - 生成图片缩略图
func TestThumbSizeByData(t *testing.T) {
	data, _ := os.ReadFile("C:\\test\\big.jpg")
	var thumb []byte
	var err error

	now := time.Now()
	for i := 0; i < 100; i++ {
		thumb, err = ThumbSizeByData(data, 200, 200, 85)
	}
	fmt.Println(time.Now().Sub(now).Seconds())
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("C:\\test\\out-go.jpg", thumb, os.ModePerm)
}

// TestThumbByData - 生成图片缩略图
func TestThumbByJpg(t *testing.T) {
	data, _ := os.ReadFile("C:\\test\\big.jpg")
	var thumb []byte
	var err error
	now := time.Now()
	for i := 0; i < 100; i++ {
		thumb, err = ThumbByJpg(data, 300)
	}
	fmt.Println(time.Now().Sub(now).Seconds())
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("C:\\test\\out-ffmpeg.jpg", thumb, os.ModePerm)
}
