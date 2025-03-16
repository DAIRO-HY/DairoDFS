package JpegUtil

import (
	"DairoDFS/util/ImageUtil/HeicUtil"
	"fmt"
	"testing"
)

func TestGetInfo2(t *testing.T) {
	data, err := HeicUtil.ToJpeg("/Users/zhoulq/dev/java/idea/DairoDFS/data/test/1.jpeg", 100)
	if err != nil {
		panic(err)
	}
	info, _ := GetInfoByData2(data)
	fmt.Println(*info)
}
