package RawUtil

import (
	application "DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {
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

func TestToPng(t *testing.T) {
	data, err := ToPng("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.png", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("C:\\Users\\user\\Desktop\\dairo-dfs-test\\bb.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.jpg", data, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	data, err := GetInfo("./data/test.cr3")
	if err != nil {
		t.Error(err)
		return
	}
	fmt.Println(data)
}
