package VideoUtil

import (
	"DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	application.FfmpegPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffmpeg"
}

func TestToPng(t *testing.T) {
	data, err := ToPng("C:\\test\\1.mov")
	if err != nil {
		t.Error(err)
		return
	}
	data, err = ImageUtil.ToJpgByData(data, 100)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\1.mov.jpg", data, os.ModePerm)
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
