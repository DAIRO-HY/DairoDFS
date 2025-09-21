package HeicUtil

import (
	"DairoDFS/extension/String"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"os"
)

// 转换成png图片
func ToPng(path string) ([]byte, error) {
	return ShellUtil.ExecToOkData("magick \"" + path + "\" PNG:-")
}

// 转换成JPEG图片
func ToJpg(path string) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ToJpgByData(data)
}

// 转换成JPEG图片
func ToJpgByData(data []byte) ([]byte, error) {
	return ShellUtil.ExecToOkData2("magick - -quality "+String.ValueOf(100)+" JPEG:-", data)
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return GetInfoByData(data)
}

// 获取图片信息
func GetInfoByData(data []byte) (ImageUtil.ImageInfo, error) {
	data, err := ToJpgByData(data)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
