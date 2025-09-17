package RawUtil

import (
	application "DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.FfmpegPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\ffmpeg"
	application.LIBRAW_BIN = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\libraw\\LibRaw-0.21.2\\bin"
	application.ExiftoolPath = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\exiftool"
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:/test/1.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	data, _ = ImageUtil.ToJpgByData(data, 80)
	os.WriteFile("C:/test/1.cr3.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("C:/test/1.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
