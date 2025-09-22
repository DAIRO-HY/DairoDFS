package DliveUtil

import (
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ImageUtil/HeicUtil"
	"DairoDFS/util/ShellUtil"
	"os"
	"strconv"
	"strings"
)

// DliveInfo 实况照片信息
type DliveInfo struct {

	//照片格式
	PhotoExt string

	//照片数据
	PhotoData []byte

	//照片文件大小
	PhotoSize int

	//视频格式
	VideoExt string

	//视频数据
	VideoData []byte

	//视频文件大小
	VideoSize int
}

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
func ToVideo(path string) []byte {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil
	}
	return ToVideoByData(data)
}

// 转换成JPEG图片
func ToJpgByData(data []byte) ([]byte, error) {
	dInfo := GetDliveInfoByData(data)
	if dInfo.PhotoExt == "heic" {
		return HeicUtil.ToJpgByData(dInfo.PhotoData)
	} else if dInfo.PhotoExt == "jpg" {
		return dInfo.PhotoData, nil
	} else {
		return nil, nil
	}
}

// 转换成JPEG图片
func ToVideoByData(data []byte) []byte {
	dInfo := GetDliveInfoByData(data)
	return dInfo.VideoData
}

// 获取实况照片信息
func GetDliveInfo(path string) DliveInfo {
	data, err := os.ReadFile(path)
	if err != nil {
		return DliveInfo{}
	}
	return GetDliveInfoByData(data)
}

// 获取实况照片信息
func GetDliveInfoByData(data []byte) DliveInfo {
	var headEndIndex int
	for i, b := range data {
		if b == 0x2D { //这是一个减号,说明头部读取结束
			headEndIndex = i
			break
		}
	}
	head := string(data[:headEndIndex])
	headArr := strings.Split(head, "|")

	var info DliveInfo
	info.PhotoExt = headArr[1]                   //图片格式
	info.PhotoSize, _ = strconv.Atoi(headArr[2]) //图片文件大小
	info.PhotoData = data[headEndIndex+1 : headEndIndex+info.PhotoSize+1]
	info.VideoExt = headArr[3]
	info.VideoSize = len(data) - info.PhotoSize - len(head) - 1
	info.VideoData = data[len(data)-info.VideoSize:]
	return info
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToJpg(path)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
