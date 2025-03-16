package JpegUtil

import (
	"DairoDFS/util/ImageUtil/HeicUtil"
	"fmt"
	"testing"
)

func TestGetInfo2(t *testing.T) {
	data, err := HeicUtil.ToJpeg("/Users/zhoulq/dev/java/idea/DairoDFS/data/test/1.heic", 1)
	if err != nil {
		panic(err)
	}
	info, _ := GetInfoByData(data)
	fmt.Println(info)
}
