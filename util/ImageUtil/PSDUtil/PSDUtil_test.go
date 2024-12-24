package PSDUtil

import (
	application "DairoDFS/appication"
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	application.FfmpegPath = "ffmpeg"
}

func TestThumb(t *testing.T) {
	data, err := Thumb("./data/logo.psd", 100, 100)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.jpg", data, os.ModePerm)
}

func TestToPng(t *testing.T) {
	data, err := ToPng("./data/logo.psd")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.png", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("./data/logo.psd")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
