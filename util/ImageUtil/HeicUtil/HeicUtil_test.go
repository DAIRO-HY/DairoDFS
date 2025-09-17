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
	data, err := ToJpg("C:\\test\\1758028265210466.heic", 2)
	if err != nil {
		panic(err)
	}
	os.WriteFile("C:\\test\\1758028265210466.heic.jpg", data, 0644)
}
