package VideoUtil

import (
	"DairoDFS/application"
	"DairoDFS/extension/File"
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

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\test\\1.mov")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\test\\1.mov.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	info, err := GetInfo("C:\\test\\1.mov")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(info.Duration)
}

func TestTransfer(t *testing.T) {
	err := Transfer(TransferArgument{
		Input:  "C:\\test\\mov\\2.mov",
		Crf:    22,
		Output: "C:\\test\\mov\\2.HDR.mp4",
	})
	if err != nil {
		t.Error(err)
		return
	}
}

func TestTransfer2(t *testing.T) {
	err := Transfer(TransferArgument{
		Input:       "C:\\test\\mov\\SDR.mp4",
		Start:       1,
		Time:        2,
		Width:       360,
		Height:      200,
		Fps:         10,
		DeleteSound: true,
		Crf:         51,
		Output:      "C:\\test\\mov\\2.SDR.SDR.mp4",
	})
	if err != nil {
		t.Error(err)
		return
	}
}

func TestHDR2SDR(t *testing.T) {
	err := HDR2SDR(TransferArgument{
		Input:       "C:\\test\\mov\\2.mov",
		DeleteSound: true,
		Crf:         22,
		Output:      "C:\\test\\mov\\2.SDR.mp4",
	})
	if err != nil {
		t.Error(err)
		return
	}
}
func TestIsHDR(t *testing.T) {
	isHDR := IsHDR("C:/test/mov/2.mov")
	fmt.Println(isHDR)
}

func TestIsHDR123(t *testing.T) {
	fmt.Println(File.ToMd5("./data/input.mov"))
}
