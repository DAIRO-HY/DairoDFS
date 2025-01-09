package PSDUtil

import (
	application "DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
)

/**
 * 生成图片缩略图
 */
func Thumb(path string, maxWidth int, maxHeight int) ([]byte, error) {
	pngData, err := ToPng(path)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByData(pngData, maxWidth, maxHeight)
}

/**
 * 生成PNG图片
 * -f image2 指定输出通用图片
 * -vcodec png指定输出图片格式为png
 */
func ToPng(path string) ([]byte, error) {
	okData, cmdErr := ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "\" -i " + path + " -f image2 -vcodec png -")
	if cmdErr != nil { //如果发生了异常，异常信息记录在了错误流数据中
		return nil, cmdErr
	}
	return okData, nil
}

/**
 * 获取图片信息
 */
func GetInfo(path string) (*ImageUtil.ImageInfo, error) {
	pngData, err := ToPng(path)
	if err != nil {
		return nil, err
	}
	return ImageUtil.GetInfoByData(pngData)
}
