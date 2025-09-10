package HeicUtil

import (
	"DairoDFS/extension/String"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
)

// 转换成png图片
func ToPng(path string) ([]byte, error) {
	return ShellUtil.ExecToOkData("magick \"" + path + "\" PNG:-")
}

// 转换成JPEG图片
func ToJpg(path string, quality int) ([]byte, error) {
	return ShellUtil.ExecToOkData("magick \"" + path + "\" -quality " + String.ValueOf(quality) + " JPEG:-")
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToJpg(path, 1)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
