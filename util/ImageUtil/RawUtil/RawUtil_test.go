package RawUtil

import (
	application "DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
	application.LIBRAW_BIN = "C:\\develop\\project\\idea\\DairoDFS\\data\\libraw\\LibRaw-0.21.2\\bin"
	application.ExiftoolPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\exiftool"
}

func TestThumb(t *testing.T) {
	data, err := Thumb("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3", 6000)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.thumb.jpg", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\Users\\user\\Desktop\\dairo-dfs-test\\raw\\tt.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\raw\\tt.cr3.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("./data/test.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
