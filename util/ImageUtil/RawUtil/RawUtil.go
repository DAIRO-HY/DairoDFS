package RawUtil

import (
	"DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"runtime"
	"strings"

	_ "golang.org/x/image/tiff"
)

/**
 * Raw图片解析工具类
 */

/**
 * 生成缩略图
 * @param path 文件路径
 * @param ext 文件后最，raw图片处理时，必须携带文件后缀名
 * @param maxWidth 图片最大宽度
 * @param maxHeight 图片最大高度
 * @return 图片字节数组
 */
func Thumb(path string, targetMaxSize int) ([]byte, error) {
	jpgData, err := ToJpg(path)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByJpg(jpgData, targetMaxSize)
}

// 从CR3文件中提取JPEG预览图
// path CR3文件路径
// @return 图片数据
func ToJpg(path string) ([]byte, error) {
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "exiftool"
	} else if runtime.GOOS == "darwin" {
		cmd = "exiftool"
	} else {
		cmd = "\"" + application.ExiftoolPath + "/exiftool-13.35_64/exiftool\""
	}
	jpgData, getJegDataErr := ShellUtil.ExecToOkData(cmd + " -b -JpgFromRaw \"" + path + "\"")
	if getJegDataErr != nil {
		return nil, getJegDataErr
	}
	orientation, getOrientationErr := ShellUtil.ExecToOkResult(cmd + " -Orientation \"" + path + "\"")
	if getOrientationErr != nil {
		return nil, getOrientationErr
	}
	orientation = strings.ReplaceAll(orientation, " ", "")
	orientation = strings.ReplaceAll(orientation, "\r", "")
	orientation = strings.ReplaceAll(orientation, "\n", "")
	if orientation == "Orientation:Rotate90CW" { //需要顺时针旋转90°
		return ImageUtil.TransposeToJpeg(jpgData, 1)
	} else if orientation == "Orientation:Rotate180CW" { //需要顺时针旋转180°
		return ImageUtil.TransposeToJpeg(jpgData, 2)
	} else if orientation == "Orientation:Rotate270CW" { //需要顺时针旋转270°
		return ImageUtil.TransposeToJpeg(jpgData, 3)
	} else {
		return jpgData, nil
	}
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "dcraw_emu"
	} else {
		cmd = "\"" + application.LIBRAW_BIN + "/dcraw_emu\""
	}

	//将图片转换成TIFF图片
	tiffData, _ := ShellUtil.ExecToOkData(cmd + " -T -Z - -mem -mmap \"" + path + "\"")
	return ImageUtil.GetInfoByData(tiffData)
}
