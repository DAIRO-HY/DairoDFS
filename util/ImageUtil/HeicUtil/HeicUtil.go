package HeicUtil

import (
	"DairoDFS/extension/String"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
)

// 生成图片缩略图
func Thumb(path string, targetMaxSize int) ([]byte, error) {
	pngData, err := ToJpeg(path, 100)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByPng(pngData, targetMaxSize)
}

// 转换成png图片
func ToPng(path string) ([]byte, error) {
	return ShellUtil.ExecToOkData("magick \"" + path + "\" PNG:-")
}

// 转换成JPEG图片
func ToJpeg(path string, quality int8) ([]byte, error) {
	return ShellUtil.ExecToOkData("magick \"" + path + "\" -quality " + String.ValueOf(quality) + " JPEG:-")
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToJpeg(path, 1)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
