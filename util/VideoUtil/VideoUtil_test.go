package VideoUtil

import (
	application "DairoDFS/appication"
	"fmt"
	"os"
	"testing"
)

func init() {

	//ffmped安装目录
	application.FfmpegPath = "ffmpeg"
	application.FfprobePath = "ffprobe"
}

func TestThumb(t *testing.T) {
	data, err := Thumb("./data/test.mp4", 300, 300)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	info, err := GetInfo("./data/test.mp4")
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
