package HeicUtil

import (
	"DairoDFS/application"
	"os"
	"testing"
)

func init() {
	application.Init()
	application.FfmpegPath = "C:\\develop\\project\\idea\\DairoDFS\\data\\ffmpeg"
}

func TestToJpegByWindows(t *testing.T) {
	data, err := ToJpeg("C:\\Users\\user\\Desktop\\dairo-dfs-test\\heic\\hh.heic", 2)
	if err != nil {
		panic(err)
	}
	os.WriteFile("C:\\Users\\user\\Desktop\\dairo-dfs-test\\heic\\hh-dfs.png.2.jpg", data, 0644)
}
