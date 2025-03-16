package JpegUtil

import (
	"DairoDFS/util/ImageUtil"
	"bytes"
	"fmt"
	"github.com/rwcarlsen/goexif/exif"
	"strconv"
)

// 获取照片信息
func GetInfoByData(data []byte) (ImageUtil.ImageInfo, error) {
	var imageInfo ImageUtil.ImageInfo

	// 解析 EXIF 数据
	x, err := exif.Decode(bytes.NewReader(data))
	if err != nil {
		return imageInfo, err
	}

	// 获取拍摄时间
	datetime, err := x.DateTime()
	if err == nil {
		imageInfo.Date = 111
		fmt.Println("拍摄时间:", datetime)
	}

	// 获取相机制造商
	manufacturer, err := x.Get(exif.Make)
	if err == nil {
		imageInfo.Make = manufacturer.String()
	}

	// 获取相机型号
	model, err := x.Get(exif.Model)
	if err == nil {
		imageInfo.Make = model.String()
	}

	// 获取光圈值
	aperture, err := x.Get(exif.FNumber)
	if err == nil {
		imageInfo.FNumber = aperture.String()
	}

	// 获取快门速度
	shutterSpeed, err := x.Get(exif.ShutterSpeedValue)
	if err == nil {
		imageInfo.ShutterSpeed = shutterSpeed.String()
	}

	// 获取 ISO
	iso, err := x.Get(exif.ISOSpeedRatings)
	if err == nil {
		imageInfo.ISO, _ = strconv.Atoi(iso.String())
	}

	// 获取 GPS 信息（如果有）
	lat, long, err := x.LatLong()
	if err == nil {
		imageInfo.Lat = lat
		imageInfo.Long = long
	}
	return imageInfo, nil
}
