package JpegUtil

import (
	"DairoDFS/util/ImageUtil"
	"bytes"
	"fmt"
	"github.com/rwcarlsen/goexif/exif"
	"log"
)

/**
 * 获取图片信息
 */
func GetInfoByData2(data []byte) (*ImageUtil.ImageInfo, error) {

	// 解析 EXIF 数据
	x, err := exif.Decode(bytes.NewReader(data))
	if err != nil {
		log.Fatal(err)
	}

	// 获取拍摄时间
	datetime, err := x.DateTime()
	if err == nil {
		fmt.Println("拍摄时间:", datetime)
	}

	// 获取相机制造商
	manufacturer, err := x.Get(exif.Make)
	if err == nil {
		fmt.Println("相机制造商:", manufacturer)
	}

	// 获取相机型号
	model, err := x.Get(exif.Model)
	if err == nil {
		fmt.Println("相机型号:", model)
	}

	// 获取光圈值
	aperture, err := x.Get(exif.FNumber)
	if err == nil {
		fmt.Println("光圈值:", aperture.String())
	}

	// 获取快门速度
	shutterSpeed, err := x.Get(exif.ShutterSpeedValue)
	if err == nil {
		fmt.Println("快门速度:", shutterSpeed.String())
	}

	// 获取 ISO
	iso, err := x.Get(exif.ISOSpeedRatings)
	if err == nil {
		fmt.Println("ISO:", iso.String())
	}

	// 获取 GPS 信息（如果有）
	lat, long, err := x.LatLong()
	if err == nil {
		fmt.Printf("GPS 坐标: 纬度 %.6f, 经度 %.6f\n", lat, long)
	}
	return nil, nil
}
