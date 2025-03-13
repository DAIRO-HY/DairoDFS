package HeicUtil

import (
	"os"
	"testing"
)

//func TestThumb(t *testing.T) {
//	data, err := Thumb("C:\\Users\\user\\Desktop\\test\\IMG_2481-1741001867627.heic", 500)
//	if err != nil {
//		t.Fatal(err)
//	}
//	fmt.Println(len(data))
//}

func TestToJpegByWindows(t *testing.T) {
	data, err := toJpegByWindows("C:\\Users\\user\\Desktop\\test\\heic-linux-diff\\1.heic", 100)
	if err != nil {
		panic(err)
	}
	os.WriteFile("./asda.jpeg", data, 0644)
}
