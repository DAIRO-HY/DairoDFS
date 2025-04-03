package VideoUtil

import (
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	//application.FfmpegPath = "./data/ffmpeg"
}

func TestThumb(t *testing.T) {
	data, err := ThumbPng("C:\\Users\\user\\Desktop\\新しいフォルダー\\tt.mov", 3840)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.png", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	info, err := GetInfo("C:\\Users\\user\\Desktop\\dairo-dfs-test\\mov\\mm.mov")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(info)
}

func TestTransfer(t *testing.T) {
	err := Transfer("./data/test.mp4", 320, 240, 15, "./data/target.mp4")
	if err != nil {
		t.Error(err)
		return
	}
}
