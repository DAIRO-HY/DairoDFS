package RawUtil

import (
	"DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"runtime"
	"strings"

	_ "golang.org/x/image/tiff"
)

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

	//获取cr3文件内嵌了哪些图片
	includeImageInfo, includedImageInfoErr := ShellUtil.ExecToOkResult(cmd + " -preview:all \"" + path + "\"")
	if includedImageInfoErr != nil {
		return nil, includedImageInfoErr
	}
	var jpgData []byte
	var getJegDataErr error
	if strings.Contains(includeImageInfo, "Jpg From Raw") { //这个文件内嵌了一张大图
		jpgData, getJegDataErr = ShellUtil.ExecToOkData(cmd + " -b -JpgFromRaw \"" + path + "\"")
	} else if strings.Contains(includeImageInfo, "Preview Image") { //这个文件内嵌了一张尺寸较小的预览图
		jpgData, getJegDataErr = ShellUtil.ExecToOkData(cmd + " -b -PreviewImage \"" + path + "\"")
	} else if strings.Contains(includeImageInfo, "Thumbnail Image") { //这个文件内嵌了一张缩略图
		jpgData, getJegDataErr = ShellUtil.ExecToOkData(cmd + " -b -ThumbnailImage \"" + path + "\"")
	} else {
		return nil, exception.Biz("这个文件没有内置jpg图片")
	}
	if len(jpgData) == 0 || getJegDataErr != nil { //极端情况，使用libraw获取jpg图片
		tiffData, err := ToTiff(path)
		if err == nil {
			jpgData, getJegDataErr = ImageUtil.ToJpgByData(tiffData, 100)
		} else {
			getJegDataErr = err
		}
	}

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

// / 将cr3转Tiff图片
func ToTiff(path string) ([]byte, error) {
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "dcraw_emu"
	} else {
		cmd = "\"" + application.LIBRAW_BIN + "/dcraw_emu\""
	}

	//将图片转换成TIFF图片
	return ShellUtil.ExecToOkData(cmd + " -T -Z - -mem -mmap \"" + path + "\"")
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
