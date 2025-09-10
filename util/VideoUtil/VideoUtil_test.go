package VideoUtil

import (
	"DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	application.FfmpegPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffmpeg"
	application.FfprobePath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffprobe"
	application.TEMP_PATH = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\temp"
}

func TestToPng(t *testing.T) {
	data, err := ToPng("C:\\test\\mov\\2.mov")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\mov\\2.mov.png", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\test\\mov\\2.mov")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\mov\\2.jpg", data, os.ModePerm)
}

func TestToJpgFromHDR(t *testing.T) {
	data, err := ToJpgFromHDR("C:\\test\\mov\\2.mov")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\mov\\2.mov.jpg", data, os.ModePerm)
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
	err := Transfer("C:/test/mov/2.mov", 0, 0, 0, "C:/test/mov/2.HDR.mp4")
	if err != nil {
		t.Error(err)
		return
	}
}

func TestTransfer2(t *testing.T) {
	err := Transfer("C:/test/mov/2.SDR.mp4", 1280, 720, 10, "C:/test/mov/2.SDR.SDR.mp4")
	if err != nil {
		t.Error(err)
		return
	}
}

func TestHDR2SDR(t *testing.T) {
	err := HDR2SDR("C:\\test\\mov\\2.mov", 0, 0, 0, "C:\\test\\mov\\2.SDR.mp4")
	if err != nil {
		t.Error(err)
		return
	}
}
func TestIsHDR(t *testing.T) {
	isHDR := IsHDR("C:/test/mov/2.mov")
	fmt.Println(isHDR)
}
