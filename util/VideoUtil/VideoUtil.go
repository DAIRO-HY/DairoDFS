package VideoUtil

import (
	application "DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"time"
)

/**
 * 生成视频缩略图
 * @param path 视频文件路径
 * @param maxWidth 图片最大宽度
 * @param maxHeight 图片最大高度
 * @return 图片字节数组
 */
func Thumb(path string, maxWidth int, maxHeight int) ([]byte, error) {

	//获取视频第一帧作为缩略图
	jpgData, cmdErr := ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "\" -i \"" + path + "\" -vf select=eq(n\\,0) -q:v 1 -f image2pipe -vcodec mjpeg -")
	if cmdErr != nil {
		return nil, cmdErr
	}
	return ImageUtil.ThumbByData(jpgData, maxWidth, maxHeight)
}

/**
 * 获取视频信息
 * @param path 视频文件路径
 * @return 图片字节数组
 */
func GetInfo(path string) (*VedioInfo, error) {
	_, videoInfoStr, cmdErr := ShellUtil.ExecToOkAndErrorResult("\"" + application.FfprobePath + "\" -i \"" + path + "\"")
	if cmdErr != nil {
		return nil, cmdErr
	}

	//时长

	duration := func() int64 {
		defer application.StopRuntimeError() // 防止程序终止
		durationStr := regexp.MustCompile("Duration: \\d{2}:\\d{2}:\\d{2}\\.\\d{2}").FindAllString(videoInfoStr, -1)[0]
		durationStr = durationStr[10:]
		durationArr := strings.Split(durationStr, ":")

		h, _ := strconv.ParseFloat(durationArr[0], 32)
		m, _ := strconv.ParseFloat(durationArr[1], 32)
		s, _ := strconv.ParseFloat(durationArr[2], 32)
		return int64(h*60*60+m*60+s) * 1000
	}()

	//创建时间
	date := func() int64 {
		defer application.StopRuntimeError() // 防止程序终止
		dateStr := regexp.MustCompile("creation_time   \\: .*\\.").FindAllString(videoInfoStr, -1)[0]
		dateStr = dateStr[strings.Index(dateStr, ":")+1 : len(dateStr)-1]
		dateStr = strings.TrimSpace(dateStr)
		dateStr = strings.ReplaceAll(dateStr, "T", " ")
		date, _ := time.Parse("2006-01-02 15:04:05", dateStr)
		return date.UnixMilli()
	}()

	//视频比特率
	bitrate := func() int {
		defer application.StopRuntimeError() // 防止程序终止
		bitrateStr := regexp.MustCompile("\\d+ kb/s,.*fps").FindAllString(videoInfoStr, -1)[0]
		bitrateStr = bitrateStr[:strings.Index(bitrateStr, "kb")-1]
		bitrate, _ := strconv.Atoi(bitrateStr)
		return bitrate
	}()

	//帧率
	fps := func() float32 {
		defer application.StopRuntimeError() // 防止程序终止
		fpsStr := regexp.MustCompile("\\d+ kb/s,.*fps").FindAllString(videoInfoStr, -1)[0]
		fpsStr = fpsStr[strings.Index(fpsStr, ",")+2 : len(fpsStr)-4]
		fps, _ := strconv.ParseFloat(fpsStr, 32)
		return float32(fps)
	}()

	//视频宽高
	width, height := func() (int, int) {
		defer application.StopRuntimeError() // 防止程序终止
		whStr := regexp.MustCompile("Stream.+Video:.+, \\d+x\\d+").FindAllString(videoInfoStr, -1)[0]
		whStr = regexp.MustCompile(", \\d+x\\d+").FindAllString(whStr, -1)[0]
		whStr = regexp.MustCompile("\\d+x\\d+").FindAllString(whStr, -1)[0]

		widthStr := strings.Split(whStr, "x")[0]
		heightStr := strings.Split(whStr, "x")[1]
		width, _ := strconv.Atoi(widthStr)
		height, _ := strconv.Atoi(heightStr)
		return width, height
	}()

	//音频格式
	audioFormat := func() string {
		defer application.StopRuntimeError() // 防止程序终止
		audioFormatStr := regexp.MustCompile("Audio: [A-z,0-9]+").FindAllString(videoInfoStr, -1)[0]
		return audioFormatStr[7:]
	}()

	//音频采样率
	audioSampleRate := func() int {
		defer application.StopRuntimeError() // 防止程序终止
		audioSamplerateStr := regexp.MustCompile("Audio: .* Hz").FindAllString(videoInfoStr, -1)[0]
		audioSamplerateStr = regexp.MustCompile("\\d+ Hz").FindAllString(audioSamplerateStr, -1)[0]
		audioSampleRateStr := audioSamplerateStr[:len(audioSamplerateStr)-3]
		audioSampleRate, _ := strconv.Atoi(audioSampleRateStr)
		return audioSampleRate
	}()

	//音频比特率
	audioBitrate := func() int {
		defer application.StopRuntimeError() // 防止程序终止
		audioBitrateStr := regexp.MustCompile("Audio: .*\\d+ kb/s").FindAllString(videoInfoStr, -1)[0]
		audioBitrateStr = regexp.MustCompile("\\d+ kb/s").FindAllString(audioBitrateStr, -1)[0]
		audioBitrateStr = audioBitrateStr[:len(audioBitrateStr)-5]
		audioBitrate, _ := strconv.Atoi(audioBitrateStr)
		return audioBitrate
	}()
	return &VedioInfo{
		Width:           width,           //宽
		Height:          height,          //高
		Fps:             fps,             //帧数
		Bitrate:         bitrate,         //视频比特率
		Duration:        duration,        //视频时长（毫秒）
		Date:            date,            //视频创建时间戳
		AudioBitrate:    audioBitrate,    //音频比特率
		AudioSampleRate: audioSampleRate, //音频采样率（HZ）
		AudioFormat:     audioFormat,     //音频格式
	}, nil
}

/**
 * 视频转码
 */
func Transfer(path string, targetW int, targetH int, targetFps float32, targetPath string) error {
	_, errResult, cmdErr := ShellUtil.ExecToOkAndErrorResult(fmt.Sprintf("\"%s\" -i \"%s\" -vf scale=%d:%d -r %f -f mp4 \"%s\"", application.FfmpegPath, path, targetW, targetH, targetFps, targetPath))
	if cmdErr != nil {
		return exception.Biz(errResult)
	}
	return nil
}
