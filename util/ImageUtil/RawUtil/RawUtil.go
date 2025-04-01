package RawUtil

import (
	application "DairoDFS/application"
	"DairoDFS/extension/String"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/RamDiskUtil"
	"DairoDFS/util/ShellUtil"
	_ "golang.org/x/image/tiff"
	"os"
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
	return ImageUtil.ThumbByTiff(tiffData, targetMaxSize)
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

	tempFile := RamDiskUtil.GetRamFolder() + "/" + String.MakeRandStr(16)

	//先将数据写入到硬盘，因为ffmpeg无法识别tiff输入流
	if err := os.WriteFile(tempFile, tiffData, 0644); err != nil {
		return nil, err
	}
	defer os.Remove(tempFile)

	//获取视频第一帧作为缩略图
	//-q:v代表输出图片质量，取值返回2-31，2为质量最佳
	return ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -f image2pipe -vcodec tiff -i \"" + tempFile + "\"" + " -q:v 2 -f image2pipe -vcodec mjpeg -")
}

// 获取图片信息
func GetInfo(path string) (ImageUtil.ImageInfo, error) {
	data, err := ToTiff(path)
	if err != nil {
		return ImageUtil.ImageInfo{}, err
	}
	return ImageUtil.GetInfoByData(data)
}
