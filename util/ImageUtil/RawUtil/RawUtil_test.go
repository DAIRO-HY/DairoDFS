package RawUtil

import (
	application "DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
	application.LIBRAW_BIN = "C:\\develop\\project\\idea\\DairoDFS\\data\\libraw\\LibRaw-0.21.2\\bin"
}

func TestThumb(t *testing.T) {
	data, err := Thumb("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3", 6000)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.thumb.jpg", data, os.ModePerm)
}

func TestToTiff(t *testing.T) {
	data, err := ToTiff("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.tiff", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.jpg", data, os.ModePerm)
}

func TestThumbByTiff(t *testing.T) {
	application.Init()
	tiffData, _ := ToTiff("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3")
	thumbData, err := ImageUtil.ThumbByTiff(tiffData, 300)
	if err != nil {
		fmt.Println(err)
		return
	}
	os.WriteFile("./data/thumb3.jpg", thumbData, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("./data/test.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
