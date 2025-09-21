package HeicUtil

import (
	"DairoDFS/application"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.Init()
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
}

func TestToJpgByData(t *testing.T) {
	data, _ := os.ReadFile("./data/IMG_3763.HEIC")
	jpgData, _ := ToJpgByData(data)
	os.WriteFile("./data/xxx.jpg", jpgData, os.ModePerm)
}

func TestGetInfoByData(t *testing.T) {
	data, _ := os.ReadFile("./data/IMG_3763.HEIC")
	info, _ := GetInfoByData(data)
	fmt.Println(info)
}

func TestToJpegByWindows(t *testing.T) {
	data, err := ToJpg("C:\\test\\1758028265210466.heic")
	if err != nil {
		panic(err)
	}
	os.WriteFile("C:\\test\\1758028265210466.heic.jpg", data, 0644)
}
