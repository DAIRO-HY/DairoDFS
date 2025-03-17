package HeicUtil

import (
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"fmt"
)

// 生成图片缩略图
func Thumb(path string, targetMaxSize int) ([]byte, error) {
	data, err := ToJpeg(path, 100)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByData(data, targetMaxSize)
}

// 转换成JPEG图片
func ToJpeg(path string, quality int8) ([]byte, error) {
	return ShellUtil.ExecToOkData(fmt.Sprintf("magick \""+path+"\" -quality %d JPEG:-", quality))
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToJpeg(path, 1)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
