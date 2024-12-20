package ImageUtil

import (
	"bytes"
	"github.com/nfnt/resize"
	_ "golang.org/x/image/bmp"
	_ "golang.org/x/image/tiff"
	_ "golang.org/x/image/webp"
	"image"
	"image/jpeg"
	_ "image/jpeg" // 必须导入具体的图片格式
	_ "image/png"
	"os"
)

/**
* 生成图片缩略图
 */
func ThumbByFile(path string, maxWidth int, maxHeight int) ([]byte, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ThumbByData(data, maxWidth, maxHeight)
}

/**
 * 生成图片缩略图
 */
func ThumbByData(data []byte, maxWidth int, maxHeight int) ([]byte, error) {

	//加载
	imageConfig, _, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}

	//目标图片宽高比
	whTargetScale := float64(maxWidth) / float64(maxHeight)

	//输入图片宽高比
	whInputtScale := float64(imageConfig.Width) / float64(imageConfig.Height)

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

/**
 * 获取图片信息
 */
func GetInfo(path string) (*ImageInfo, error) {
	file, _ := os.Open(path)
	defer file.Close()

	//加载
	imageConfig, _, err := image.DecodeConfig(file)
	if err != nil {
		return nil, err
	}
	return &ImageInfo{
		Width:  imageConfig.Width,
		Height: imageConfig.Height,
	}, nil
}
