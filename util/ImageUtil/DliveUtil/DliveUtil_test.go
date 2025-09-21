package DliveUtil

import (
	"DairoDFS/application"
	"DairoDFS/extension/File"
	"fmt"
	"os"
	"testing"
)

func init() {
	application.Init()
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
}

func TestToJpg(t *testing.T) {
	jpgData, _ := ToJpg("./data/jpg.dlive")
	os.WriteFile("./data/xxx.jpg", jpgData, os.ModePerm)
}

func TestToVideo(t *testing.T) {
	videoData := ToVideo("./data/jpg.dlive")
	os.WriteFile("./data/xxx.mov", videoData, os.ModePerm)
}

func TestGetInfo(t *testing.T) {
	info, _ := GetInfo("./data/heic.dlive")
	fmt.Println(info)
}

func TestGetInfotemp(t *testing.T) {
	fmt.Println(File.ToMd5("./data/xxx.jpg"))
	fmt.Println(File.ToMd5("./data/before.jpg"))
}
