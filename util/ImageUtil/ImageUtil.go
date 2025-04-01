package ImageUtil

import (
	"DairoDFS/application"
	"DairoDFS/extension/String"
	"DairoDFS/util/RamDiskUtil"
	"DairoDFS/util/ShellUtil"
	"bytes"
	"github.com/nfnt/resize"
	"github.com/rwcarlsen/goexif/exif"
	_ "golang.org/x/image/bmp"
	_ "golang.org/x/image/tiff"
	_ "golang.org/x/image/webp"
	"image"
	"image/color"
	"image/draw"
	"image/jpeg"
	_ "image/png"
	"os"
	"strconv"
)

/**
* 生成图片缩略图
 */
func ThumbByFile(path string, targetMaxSize int) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ThumbByData(data, targetMaxSize)
}

// 生成图片缩略图
func ThumbByData(data []byte, targetMaxSize int) ([]byte, error) {

	//加载
	imageConfig, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//目标宽,高
	targetW, targetH := GetScaleSize(imageConfig.Width, imageConfig.Height, targetMaxSize)

	//加载图片
	img, _, err := image.Decode(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//data不再使用，让GC尽快回收
	data = nil
	if format == "png" { //如果图片是png格式，将背景填充白色

		// 创建一个新的 RGBA 图像
		bounds := img.Bounds()

		//填充背景色后的图片
		pngFill := image.NewRGBA(bounds)

		// 指定填充颜色（如白色）
		fillColor := color.RGBA{R: 255, G: 255, B: 255, A: 255}

		// 填充背景颜色
		draw.Draw(pngFill, bounds, &image.Uniform{fillColor}, image.Point{}, draw.Src)

		// 将原始图片绘制到新图像上，保留非透明部分
		draw.Draw(pngFill, bounds, img, bounds.Min, draw.Over)
		img = pngFill
	}

	//重新设置图片尺寸
	zoomImg := resize.Resize(uint(targetW), uint(targetH), img, resize.Lanczos3)

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	////编码信息
	//options := &encoder.Options{
	//	Quality: 100,
	//}
	//// 将裁剪后的图片编码并保存
	//webp.Encode(&buf, zoomImg, options)

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: 100, // 设定 JPEG 质量 1-100
	}
	err = jpeg.Encode(&buf, zoomImg, options)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// 生成图片缩略图
func ThumbByJpg(data []byte, targetMaxSize int) ([]byte, error) {
	info, infoErr := GetInfoByData(data)
	if infoErr != nil {
		return nil, infoErr
	}

	//获取转换之后的尺寸
	targetW, targetH := GetScaleSize(info.Width, info.Height, targetMaxSize)

	//获取视频第一帧作为缩略图
	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	//v指定输出图片尺寸
	return ShellUtil.ExecToOkData2("\""+application.FfmpegPath+"/ffmpeg\" -f image2pipe -vcodec mjpeg -i pipe:0 -vf scale="+String.ValueOf(targetW)+":"+String.ValueOf(targetH)+" -q:v 3 -f image2pipe -vcodec mjpeg -", data)
}

// 生成图片缩略图
func ThumbByPng(data []byte, targetMaxSize int) ([]byte, error) {
	info, infoErr := GetInfoByData(data)
	if infoErr != nil {
		return nil, infoErr
	}

	//获取转换之后的尺寸
	targetW, targetH := GetScaleSize(info.Width, info.Height, targetMaxSize)

	//获取视频第一帧作为缩略图
	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	//v指定输出图片尺寸
	return ShellUtil.ExecToOkData2("\""+application.FfmpegPath+"/ffmpeg\" -f image2pipe -vcodec png -i pipe:0 -vf scale="+String.ValueOf(targetW)+":"+String.ValueOf(targetH)+" -q:v 3 -f image2pipe -vcodec mjpeg -", data)
}

// 生成图片缩略图
func ThumbByTiffPath(path string, targetMaxSize int) ([]byte, error) {
	tiffData, _ := os.ReadFile(path)
	return ThumbByTiff(tiffData, targetMaxSize)
}

// 生成图片缩略图
func ThumbByTiff(data []byte, targetMaxSize int) ([]byte, error) {
	tempFile := RamDiskUtil.GetRamFolder() + "/" + String.MakeRandStr(16)

	//先将数据写入到硬盘，因为ffmpeg无法识别tiff输入流
	if err := os.WriteFile(tempFile, data, 0644); err != nil {
		return nil, err
	}
	defer os.Remove(tempFile)

	info, infoErr := GetInfoByData(data)
	if infoErr != nil {
		return nil, infoErr
	}

	//获取转换之后的尺寸
	targetW, targetH := GetScaleSize(info.Width, info.Height, targetMaxSize)

	//获取视频第一帧作为缩略图
	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	//v指定输出图片尺寸
	return ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -f image2pipe -vcodec tiff -i \"" + tempFile + "\" -vf scale=" + String.ValueOf(targetW) + ":" + String.ValueOf(targetH) + " -q:v 3 -f image2pipe -vcodec mjpeg -")
}

// 将png图片转jpg
// quatity 转换质量：2-31  2为质量最佳
func Png2Jpg(data []byte, quality int8) ([]byte, error) {

	//获取视频第一帧作为缩略图
	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	//v指定输出图片尺寸
	return ShellUtil.ExecToOkData2("\""+application.FfmpegPath+"/ffmpeg\" -f image2pipe -vcodec png -i pipe:0 -q:v "+String.ValueOf(quality)+" -f image2pipe -vcodec mjpeg -", data)
}

// 按比例缩放图片
// srcWidth 原始宽
// srcWidth 原始高
// targetMaxSize 目标最大边
func GetScaleSize(srcWidth int, srcHeight int, targetMaxSize int) (int, int) {

	//输入图片宽高比
	whInputScale := float64(srcWidth) / float64(srcHeight)

	//目标宽
	var targetW int

	//目标高
	var targetH int

	if whInputScale > 1 { //这是一张横向图片
		if srcWidth <= targetMaxSize {
			return srcWidth, srcHeight
		}
		targetW = targetMaxSize
		targetH = int(float64(targetW) / whInputScale)
	} else { //这是一张竖向图片
		if srcHeight <= targetMaxSize {
			return srcWidth, srcHeight
		}
		targetH = targetMaxSize
		targetW = int(float64(targetH) * whInputScale)
	}
	return targetW, targetH
}

/**
 * 生成正方形图片缩略图
 */
func ThumbByDataToSquare(data []byte, maxWidth int, maxHeight int) ([]byte, error) {

	//加载
	imageConfig, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//输入图片宽高比
	whInputtScale := float64(imageConfig.Width) / float64(imageConfig.Height)

	//目标图片宽高比
	whTargetScale := float64(maxWidth) / float64(maxHeight)

	//裁剪宽度
	var cutWidth int

	//裁剪宽度
	var cutHeight int

	//裁剪坐标
	var x int
	var y int
	if whTargetScale > whInputtScale {
		cutWidth = imageConfig.Width
		cutHeight = int((float64(imageConfig.Width) / whTargetScale))

		x = 0
		y = (imageConfig.Height - cutHeight) / 2
	} else {
		cutWidth = int(float64(imageConfig.Height) * whTargetScale)
		cutHeight = imageConfig.Height

		x = (imageConfig.Width - cutWidth) / 2
		y = 0
	}

	// 定义裁剪区域 (x0, y0, x1, y1)
	cropRect := image.Rect(x, y, x+cutWidth, y+cutHeight)

	//加载图片
	img, _, err := image.Decode(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//data不再使用，让GC尽快回收
	data = nil
	if format == "png" { //如果图片是png格式，将背景填充白色

		// 创建一个新的 RGBA 图像
		bounds := img.Bounds()

		//填充背景色后的图片
		pngFill := image.NewRGBA(bounds)

		// 指定填充颜色（如白色）
		fillColor := color.RGBA{R: 255, G: 255, B: 255, A: 255}

		// 填充背景颜色
		draw.Draw(pngFill, bounds, &image.Uniform{fillColor}, image.Point{}, draw.Src)

		// 将原始图片绘制到新图像上，保留非透明部分
		draw.Draw(pngFill, bounds, img, bounds.Min, draw.Over)
		img = pngFill
	}

	//按比例裁切
	croppedImg := img.(interface {
		SubImage(r image.Rectangle) image.Image
	}).SubImage(cropRect)

	//img不再使用，让GC尽快回收
	img = nil

	if imageConfig.Width > maxWidth {

		//重新设置图片尺寸
		//image.resize(maxWidth, maxHeight)
		croppedImg = resize.Resize(uint(maxWidth), uint(maxHeight), croppedImg, resize.Lanczos3)
	}

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: 85, // 设定 JPEG 质量 1-100
	}

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 将裁剪后的图片编码并保存
	err = jpeg.Encode(&buf, croppedImg, options)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// 获取照片信息
func GetInfo(path string) (ImageInfo, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return ImageInfo{}, err
	}
	return GetInfoByData(data)
}

// 获取照片信息
func GetInfoByData(data []byte) (ImageInfo, error) {
	var imageInfo ImageInfo

	//加载图片信息
	imageConfig, _, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return imageInfo, err
	}

	// 设置图片尺寸
	imageInfo.Width = imageConfig.Width
	imageInfo.Height = imageConfig.Height

	// 解析 EXIF 数据
	x, err := exif.Decode(bytes.NewReader(data))
	if err != nil {
		//	return imageInfo, err

		//即使出错，也要返回已经生成的属性
		return imageInfo, nil
	}

	// 获取拍摄时间
	datetime, err := x.DateTime()
	if err == nil {
		imageInfo.Date = datetime.UnixMilli()
	}

	// 获取相机制造商
	manufacturer, err := x.Get(exif.Make)
	if err == nil {
		imageInfo.Make = manufacturer.String()
	}

	// 获取相机型号
	model, err := x.Get(exif.Model)
	if err == nil {
		imageInfo.Make = model.String()
	}

	// 获取光圈值
	aperture, err := x.Get(exif.FNumber)
	if err == nil {
		imageInfo.FNumber = aperture.String()
	}

	// 获取快门速度
	shutterSpeed, err := x.Get(exif.ShutterSpeedValue)
	if err == nil {
		imageInfo.ShutterSpeed = shutterSpeed.String()
	}

	// 获取 ISO
	iso, err := x.Get(exif.ISOSpeedRatings)
	if err == nil {
		imageInfo.ISO, _ = strconv.Atoi(iso.String())
	}

	// 获取 GPS 信息（如果有）
	lat, long, err := x.LatLong()
	if err == nil {
		imageInfo.Lat = lat
		imageInfo.Long = long
	}
	return imageInfo, nil
}
