package ImageUtil

import (
	"DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
}

/**
 * 生成图片缩略图
 */
func TestThumbByFile(t *testing.T) {
	thumb, err := ThumbByFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3", 100)
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
