package RawUtil

import (
	application "DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.LibrawPath = "C:/develop/project/idea/DairoDFS-JAVA/data/lib/libraw/LibRaw-0.21.3/bin"
}

func TestThumb(t *testing.T) {
	data, err := Thumb("./data/test.cr3", "cr3", 300, 300)
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.thumb.jpg", data, os.ModePerm)
}

func TestToTiff(t *testing.T) {
	data, err := ToTiff("./data/test.cr3", "cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.tiff", data, os.ModePerm)
}

func TestToPng(t *testing.T) {
	data, err := ToPng("./data/test.cr3", "cr3")
	if err != nil {
		t.Error(err)
		return
	}
	os.WriteFile("./data/test.png", data, os.ModePerm)
}

func TestToJpg(t *testing.T) {
	data, err := ToJpg("./data/test.cr3", "cr3")
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
