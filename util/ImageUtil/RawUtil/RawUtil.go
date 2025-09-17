package RawUtil

import (
	"DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"runtime"
	"strconv"
	"strings"
	"time"

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

	//设置旋转属性
	if info, infoErr := GetInfo(path); infoErr == nil {
		if info.Orientation != 1 {
			jpgData, _ = ImageUtil.WriteOrientation(jpgData, uint16(info.Orientation))
		}
	}
	return jpgData, nil
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
func GetInfoBk(path string) (ImageUtil.ImageInfo, error) {
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

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "exiftool"
	} else if runtime.GOOS == "darwin" {
		cmd = "exiftool"
	} else {
		cmd = "\"" + application.ExiftoolPath + "/exiftool-13.35_64/exiftool\""
	}

	//获取cr3文件内嵌了哪些图片
	rawInfoStr, infoErr := ShellUtil.ExecToOkResult(cmd + " -Orientation -ImageWidth -ImageHeight -CreateDate -ISO -FNumber -ExposureTime -Make \"" + path + "\"")
	if infoErr != nil {
		return ImageUtil.ImageInfo{}, infoErr
	}
	rawInfoStr = strings.ReplaceAll(rawInfoStr, "\r\n", "\n")
	rawInfoStr = strings.ReplaceAll(rawInfoStr, "\r", "\n")
	rawInfoMap := make(map[string]string)
	for _, it := range strings.Split(rawInfoStr, "\n") { //将属性解析到map
		if len(it) == 0 {
			continue
		}
		splitIndex := strings.Index(it, ":")
		key := it[:splitIndex]
		value := it[splitIndex+1:]
		key = strings.TrimSpace(key)
		value = strings.TrimSpace(value)
		rawInfoMap[key] = value
	}
	info := ImageUtil.ImageInfo{}
	date, dateErr := time.Parse("2006:01:02 15:04:05-07:00", rawInfoMap["Create Date"])
	if dateErr == nil { //得到拍摄时间
		info.Date = date.UnixMilli()
	}
	if width, ok := rawInfoMap["Image Width"]; ok { //得到照片宽
		info.Width, _ = strconv.Atoi(width)
	}
	if height, ok := rawInfoMap["Image Height"]; ok { //得到照片宽
		info.Height, _ = strconv.Atoi(height)
	}
	if ios, ok := rawInfoMap["ISO"]; ok { //得到曝光
		info.ISO, _ = strconv.Atoi(ios)
	}
	if fNumber, ok := rawInfoMap["F Number"]; ok { //得到光圈
		info.FNumber = fNumber
	}
	if shutterSpeed, ok := rawInfoMap["Exposure Time"]; ok { //得到快门速度
		info.ShutterSpeed = shutterSpeed
	}
	if cmake, ok := rawInfoMap["Make"]; ok { //得到相机品牌
		info.Make = cmake
	}
	if orientation, ok := rawInfoMap["Orientation"]; ok { //得到相机品牌
		if orientation == "Rotate 90 CW" { //需要顺时针旋转90°
			info.Orientation = 6
		} else if orientation == "Rotate 180 CW" { //需要顺时针旋转180°
			info.Orientation = 3
		} else if orientation == "Rotate 270 CW" { //需要顺时针旋转270°
			info.Orientation = 8
		} else {
			info.Orientation = 1
		}
	} else {
		info.Orientation = 1
	}
	return info, nil
}
