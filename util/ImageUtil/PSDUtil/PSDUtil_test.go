package PSDUtil

import (
	application "DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	application.FfmpegPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffmpeg"
}

func TestToPng(t *testing.T) {
	data, err := ToPng("C:\\test\\1.psd")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\1.psd.png", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\test\\1.psd")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\1.psd.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("./data/logo.psd")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
