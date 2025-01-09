package RawUtil

import (
	application "DairoDFS/application"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"bytes"
	"fmt"
	_ "golang.org/x/image/tiff"
	"image"
	"image/jpeg"
	"image/png"
	"os"
	"regexp"
	"strconv"
	"strings"
	"time"
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
func Thumb(path string, ext string, maxWidth int, maxHeight int) ([]byte, error) {
	tiffData, err := ToTiff(path, ext)
	if err != nil {
		return nil, err
	}
	return ImageUtil.ThumbByData(tiffData, maxWidth, maxHeight)
}

/**
 * 生成Tiff图片
 * @param path raw文件路径
 * @param ext 文件后最，raw图片处理时，必须携带文件后缀名
 * @return 图片数据
 */
func ToTiff(path string, ext string) ([]byte, error) {

	//重命名文件名
	renameTo := path + "." + ext
	renameErr := os.Rename(path, renameTo)
	if renameErr != nil {
		return nil, renameErr
	}
	defer os.Rename(renameTo, path)

	//将图片转换成TIFF图片
	okData, cmdErr :=
		ShellUtil.ExecToOkData("\"" + application.LibrawPath + "/dcraw_emu\" -T -w -Z - -mem -mmap \"" + renameTo + "\"")
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
func ToPng(path string, ext string) ([]byte, error) {
	tiffData, tiffErr := ToTiff(path, ext)
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
func ToJpg(path string, ext string) ([]byte, error) {
	tiffData, tiffErr := ToTiff(path, ext)
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

/**
 * 获取图片信息
 */
func GetInfo(path string) (*ImageUtil.ImageInfo, error) {
	okRs, cmdErr :=
		ShellUtil.ExecToOkResult("\"" + application.LibrawPath + "/raw-identify\" -v \"" + path + "\"")
	if cmdErr != nil { //如果发生了异常，异常信息记录在了错误流数据中
		return nil, cmdErr
	}

	//拍摄时间
	date := func() int64 {
		defer application.StopRuntimeError() // 防止程序终止
		durationStr := regexp.MustCompile("Timestamp:.*").FindAllString(okRs, -1)[0]
		durationStr = durationStr[11 : len(durationStr)-1]
		durationArr := strings.Split(durationStr, " ")
		montnMap := map[string]string{
			"Jan": "01",
			"Feb": "02",
			"Mar": "03",
			"Apr": "04",
			"May": "05",
			"Jun": "06",
			"Jul": "07",
			"Aug": "08",
			"Sep": "09",
			"Oct": "10",
			"Nov": "11",
			"Dec": "12",
		}
		month := montnMap[durationArr[1]]
		day, _ := strconv.Atoi(durationArr[3])
		dateStr := durationArr[5] + month + fmt.Sprintf("%02d", day) + durationArr[4]
		date, _ := time.Parse("2006010215:04:05", dateStr)
		return date.UnixMilli()
	}()

	//获取宽高
	width, height := func() (int, int) {
		imageSizeStr := regexp.MustCompile("Image size:.*").FindAllString(okRs, -1)[0]
		imageSizeStr = imageSizeStr[11 : len(imageSizeStr)-1]
		imageSizeStr = strings.ReplaceAll(imageSizeStr, " ", "")
		imageSizeArr := strings.Split(imageSizeStr, "x")
		width, _ := strconv.Atoi(imageSizeArr[0])  //宽
		height, _ := strconv.Atoi(imageSizeArr[1]) //高
		return width, height
	}()

	//相机名
	camera := func() string {
		var cameraStr = regexp.MustCompile("Camera:.*").FindAllString(okRs, -1)[0]
		return cameraStr[8 : len(cameraStr)-1]
	}()
	return &ImageUtil.ImageInfo{
		Width:  width,
		Height: height,
		Camera: camera,
		Date:   date,
	}, nil
}
