package HeicUtil

import (
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
)

// 生成图片缩略图
func Thumb(path string, targetMaxSize int) ([]byte, error) {
	pngData, err := ToPng(path)
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
	pngData, err := ToPng(path)
	if err != nil {
		return nil, err
	}
	return ImageUtil.Png2Jpg(pngData, quality)
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToPng(path)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
