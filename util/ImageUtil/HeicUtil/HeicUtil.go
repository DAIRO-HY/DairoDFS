package HeicUtil

import (
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"bytes"
	"fmt"
	"github.com/klippa-app/go-libheif"
	"image/jpeg"
	"os"
)

// 生成图片缩略图
func Thumb(path string, targetMaxSize int) ([]byte, error) {
	file, err := os.Open(path)
	img, err := libheif.DecodeImage(file)
	if err != nil {
		// Handle error.
		return nil, err
	}

	// 设置 JPEG 编码选项
	options := &jpeg.Options{
		Quality: 85, // 设定 JPEG 质量 1-100
	}

	// 创建一个 bytes.Buffer 用于保存 JPEG 数据
	var buf bytes.Buffer

	// 将裁剪后的图片编码并保存
	err = jpeg.Encode(&buf, img, options)
	return ImageUtil.ThumbByData(buf.Bytes(), targetMaxSize)
}

// 转换成JPEG图片
func ToJpeg(path string, quality int8) ([]byte, error) {
	return ShellUtil.ExecToOkData(fmt.Sprintf("magick \""+path+"\" -quality %d JPEG:-", quality))
}
