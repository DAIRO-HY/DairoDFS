package ImageUtil

import (
	"DairoDFS/application"
	"DairoDFS/util/ShellUtil"
	"bytes"
	"image"
	"image/color"
	"image/draw"
	"image/jpeg"
	_ "image/png"
	"os"
	"strconv"

	"github.com/dsoprea/go-jpeg-image-structure/v2"
	_ "golang.org/x/image/tiff" //对tiff支持

	"github.com/nfnt/resize"
	rexif "github.com/rwcarlsen/goexif/exif"
	_ "golang.org/x/image/bmp"
	_ "golang.org/x/image/webp"
)

// Resize - 裁剪图片
func Resize(path string, targetMaxSize int, quality int) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ResizeByData(data, targetMaxSize, quality)
}

// ResizeByData - 设置图片大小
func ResizeByData(data []byte, targetMaxSize int, quality int) ([]byte, error) {

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
	zoomImg := resize.Resize(uint(targetW), uint(targetH), img, resize.MitchellNetravali)

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
		Quality: quality, // 设定 JPEG 质量 1-100
	}
	err = jpeg.Encode(&buf, zoomImg, options)
	if err != nil {
		return nil, err
	}

	//得到jpg数据
	jpgData := buf.Bytes()

	//读取原图的属性，为新图设置旋转属性
	if info, infoErr := GetInfoByData(data); infoErr == nil {
		if info.Orientation != 1 {
			jpgData, _ = WriteOrientation(jpgData, uint16(info.Orientation))
		}
	}
	return jpgData, nil
}

// Crop - 裁剪图片
func Crop(path string, width int, height int, quality int) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return CropByData(data, width, height, quality)
}

// CropByData - 裁剪图片
func CropByData(data []byte, width int, height int, quality int) ([]byte, error) {

	//加载
	imageConfig, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//输入图片宽高比
	whInputtScale := float64(imageConfig.Width) / float64(imageConfig.Height)

	//目标图片宽高比
	whTargetScale := float64(width) / float64(height)

	//裁剪宽度
	var cutWidth int

	//裁剪宽度
	var cutHeight int

	//裁剪坐标
	var x int
	var y int
	if whTargetScale > whInputtScale { //目标宽高比比输入宽高比大时
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
	if cutWidth > width { //如果裁剪后的宽比目标宽大，则需要再次进行缩放

		//重新设置图片尺寸
		croppedImg = resize.Resize(uint(width), uint(height), croppedImg, resize.MitchellNetravali)
	}

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: quality, // 设定 JPEG 质量 1-100
	}

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 将裁剪后的图片编码并保存
	err = jpeg.Encode(&buf, croppedImg, options)
	if err != nil {
		return nil, err
	}

	//得到jpg数据
	jpgData := buf.Bytes()

	//读取原图的属性，为新图设置旋转属性
	if info, infoErr := GetInfoByData(data); infoErr == nil {
		if info.Orientation != 1 {
			jpgData, _ = WriteOrientation(jpgData, uint16(info.Orientation))
		}
	}
	return jpgData, nil
}

// 图片转换为Jpg
func ToJpg(path string, quality int) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ToJpgByData(data, quality)
}

// 图片转换为Jpg
func ToJpgByData(data []byte, quality int) ([]byte, error) {

	//加载
	_, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//加载图片
	img, _, err := image.Decode(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}
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

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: quality, // 设定 JPEG 质量 1-100
	}
	err = jpeg.Encode(&buf, img, options)
	if err != nil {
		return nil, err
	}

	//得到jpg数据
	jpgData := buf.Bytes()

	//读取原图的属性，为新图设置旋转属性
	if info, infoErr := GetInfoByData(data); infoErr == nil {
		if info.Orientation != 1 {
			jpgData, _ = WriteOrientation(jpgData, uint16(info.Orientation))
		}
	}
	return jpgData, nil
}

// 旋转图片并将图片转换成jpeg
// transpose 旋转角度 1：顺时针90° 2：顺时针180° 3：顺时针270°
func TransposeToJpeg(data []byte, transpose int8) ([]byte, error) {

	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	ffmpeg := "\"" + application.FfmpegPath + "/ffmpeg\""
	if transpose == 1 { //需要顺时针旋转90°
		return ShellUtil.ExecToOkData2(ffmpeg+" -i pipe:0 -vf \"transpose=1\" -q:v 2 -", data)
	} else if transpose == 2 { //需要顺时针旋转180°
		return ShellUtil.ExecToOkData2(ffmpeg+" -i pipe:0 -vf \"transpose=1,transpose=1\" -q:v 2 -", data)
	} else if transpose == 3 { //需要顺时针旋转270°
		return ShellUtil.ExecToOkData2(ffmpeg+" -i pipe:0 -vf \"transpose=1,transpose=1,transpose=1\" -q:v 2 -f image2pipe -vcodec mjpeg -", data)
	} else {
		return data, nil
	}
}

// WriteOrientation - 写入旋转属
// 1 = 正常方向
// 6 = 需要顺时针旋转90度（宽高对调）
// 8 = 逆时针旋转90度（宽高对调）
// 3 = 旋转180度
func WriteOrientation(data []byte, value uint16) ([]byte, error) {
	return WriteExif(data, "Orientation", []uint16{value})
}

// WriteExif - 往JPG文件写入属性
func WriteExif(data []byte, name string, value any) ([]byte, error) {
	// パーサーを作る
	jmp := jpegstructure.NewJpegMediaParser()

	// JPEGファイルを読み取ってセグメントリストを得る
	ec, err := jmp.ParseBytes(data)
	if err != nil {
		return nil, err
	}
	sl := ec.(*jpegstructure.SegmentList)

	// IfdBuilderを作る
	rootBuilder, err := sl.ConstructExifBuilder()
	if err != nil {
		return nil, err
	}

	err = rootBuilder.SetStandardWithName(name, value)
	if err != nil {
		return nil, err
	}

	// SegmentListを更新する
	err = sl.SetExif(rootBuilder)
	if err != nil {
		return nil, err
	}

	// 创建一个 bytes.Buffer 用于保存数据
	var buf bytes.Buffer

	// 新しいファイルに書き込む
	err = sl.Write(&buf)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
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
	x, err := rexif.Decode(bytes.NewReader(data))
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
	manufacturer, err := x.Get(rexif.Make)
	if err == nil {
		imageInfo.Make = manufacturer.String()
	}

	// 获取相机型号
	model, err := x.Get(rexif.Model)
	if err == nil {
		imageInfo.Make = model.String()
	}

	// 获取光圈值
	aperture, err := x.Get(rexif.FNumber)
	if err == nil {
		imageInfo.FNumber = aperture.String()
	}

	// 获取快门速度
	shutterSpeed, err := x.Get(rexif.ShutterSpeedValue)
	if err == nil {
		imageInfo.ShutterSpeed = shutterSpeed.String()
	}

	// 获取 ISO
	iso, err := x.Get(rexif.ISOSpeedRatings)
	if err == nil {
		imageInfo.ISO, _ = strconv.Atoi(iso.String())
	}

	// 获取图片方向
	orient, err := x.Get(rexif.Orientation)
	if err == nil {
		// Orientation 值的意义如下：
		// 1 = 正常方向
		// 6 = 需要顺时针旋转90度（宽高对调）
		// 8 = 逆时针旋转90度（宽高对调）
		// 3 = 旋转180度
		imageInfo.Orientation, _ = orient.Int(0)
	}

	// 获取 GPS 信息（如果有）
	lat, long, err := x.LatLong()
	if err == nil {
		imageInfo.Lat = lat
		imageInfo.Long = long
	}
	//if imageInfo.Orientation == 6 || imageInfo.Orientation == 8 { //这张图片有90°旋转，则调换宽高属性
	//	temp := imageInfo.Width
	//	imageInfo.Width = imageInfo.Height
	//	imageInfo.Height = temp
	//}
	return imageInfo, nil
}
