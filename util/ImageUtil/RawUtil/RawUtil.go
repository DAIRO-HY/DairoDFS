package RawUtil

import (
	application "DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"bytes"
	_ "golang.org/x/image/tiff"
	"image"
	"image/jpeg"
	"image/png"
	"runtime"
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
	tiffData, err := ToTiff(path)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByData(tiffData, targetMaxSize)
}

/**
 * 生成Tiff图片
 * @param path raw文件路径
 * @param ext 文件后最，raw图片处理时，必须携带文件后缀名
 * @return 图片数据
 */
func ToTiff(path string) ([]byte, error) {
	var cmd string
	if runtime.GOOS == "linux" {
		cmd = "dcraw_emu"
	} else {
		cmd = "\"" + application.LIBRAW_BIN + "/dcraw_emu\""
	}

	//将图片转换成TIFF图片
	okData, cmdErr :=
		ShellUtil.ExecToOkData(cmd + " -T -w -b 1.4 -g 10.4 12.92 -Z - -mem -mmap \"" + path + "\"")
	if cmdErr != nil {
		return nil, cmdErr
	}
	return okData, nil
}

/**
 * 生成PNG图片
 * @param path raw文件路径
 * @param ext 文件后最，raw图片处理时，必须携带文件后缀名
 * @return 图片数据
 */
func ToPng(path string) ([]byte, error) {
	tiffData, tiffErr := ToTiff(path)
	if tiffErr != nil {
		return nil, tiffErr
	}

	//加载图片
	tiff, _, err := image.Decode(bytes.NewReader(tiffData))
	if err != nil {
		return nil, err
	}

	// 使用 png.Encoder 指定压缩级别
	encoder := png.Encoder{
		CompressionLevel: png.BestCompression, // 可选：DefaultCompression, NoCompression, BestSpeed, BestCompression
	}

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 图片编码
	err = encoder.Encode(&buf, tiff)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

/**
 * 生成Jpg图片
 * @param path raw文件路径
 * @param ext 文件后最，raw图片处理时，必须携带文件后缀名
 * @return 图片数据
 */
func ToJpg(path string) ([]byte, error) {
	tiffData, tiffErr := ToTiff(path)
	if tiffErr != nil {
		return nil, tiffErr
	}

	//加载图片
	tiff, _, err := image.Decode(bytes.NewReader(tiffData))
	if err != nil {
		return nil, err
	}

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: 100, // 设定 JPEG 质量 1-100
	}

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 将裁剪后的图片编码并保存
	err = jpeg.Encode(&buf, tiff, options)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToTiff(path)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
