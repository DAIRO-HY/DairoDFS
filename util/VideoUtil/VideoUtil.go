package VideoUtil

import (
	application "DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/extension/String"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ShellUtil"
	"encoding/json"
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"time"
)

// 生成视频缩略图
// path 视频文件路径
// targetMaxSize 图片最大边
// return 图片字节数组,错误信息
func Thumb(path string, targetMaxSize int) ([]byte, error) {

	//获取视频第一帧作为缩略图
	jpgData, cmdErr := ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -i \"" + path + "\" -vf select=eq(n\\,0) -q:v 1 -f image2pipe -vcodec mjpeg -")
	if cmdErr != nil {
		return nil, cmdErr
	}
	return ImageUtil.ThumbByData(jpgData, targetMaxSize, 85)
}

// 生成视频缩略图
// path 视频文件路径
// tagetMaxSize 图片最大边
// return 图片字节数组,错误信息
func ThumbPng(path string, targetMaxSize int) ([]byte, error) {
	info, infoErr := GetInfo(path)
	if infoErr != nil {
		return nil, infoErr
	}

	//目标宽,高
	targetW, targetH := ImageUtil.GetScaleSize(info.Width, info.Height, targetMaxSize)

	//获取视频第一帧作为缩略图,以png格式输出。经过实际验证，输出jpg会导致颜色泛白。
	//参数说明
	//-pix_fmt rgb24  使用 标准 RGB，不带透明度（减少存储体积）
	//-pred mixed 混合预测模式，通常比默认模式压缩得更好。
	//-f image2pipe -vcodec png  强制使用png编码
	pngData, cmdErr := ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -i \"" + path + "\" -vf \"scale=" + String.ValueOf(targetW) + ":" + String.ValueOf(targetH) + ",select=eq(n\\,0)\" -q:v 1 -pix_fmt rgb24 -pred mixed -f image2pipe -vcodec png -")
	if cmdErr != nil {
		return nil, cmdErr
	}
	return pngData, nil
}

/**
 * 获取视频信息
 * @param path 视频文件路径
 * @return 图片字节数组
 */
func GetInfoBk(path string) (VedioInfo, error) {
	_, videoInfoStr, cmdErr := ShellUtil.ExecToOkAndErrorResult("\"" + application.FfprobePath + "/ffprobe\" -i \"" + path + "\"")
	if cmdErr != nil {
		return VedioInfo{}, cmdErr
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
	return VedioInfo{
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
 * 获取视频信息
 * @param path 视频文件路径
 * @return 图片字节数组
 */
func GetInfo(path string) (VedioInfo, error) {
	infoData, cmdErr := ShellUtil.ExecToOkData("\"" + application.FfprobePath + "/ffprobe\" -print_format json -show_streams \"" + path + "\"")
	if cmdErr != nil {
		return VedioInfo{}, cmdErr
	}
	infoMap := make(map[string]any)
	json.Unmarshal(infoData, &infoMap)

	streams, isOk := infoMap["streams"]
	if !isOk {
		return VedioInfo{}, cmdErr
	}
	for _, it := range streams.([]any) {
		stream := it.(map[string]any)
		if stream["codec_type"] != "video" {
			continue
		}
		info := VedioInfo{}
		info.Width = int(stream["width"].(float64))
		info.Height = int(stream["height"].(float64))
		info.Duration = int64(stream["duration_ts"].(float64))
		{
			rFrameRate := stream["r_frame_rate"].(string)
			rFrameRates := strings.Split(rFrameRate, "/")
			fm, _ := strconv.Atoi(rFrameRates[0])
			fz, _ := strconv.Atoi(rFrameRates[1])
			info.Fps = float32(fm) / float32(fz)
		}
		info.Bitrate, _ = strconv.Atoi(stream["bit_rate"].(string))
		if tags, hasTag := stream["tags"]; hasTag {
			if creationTime, hasCreationTime := tags.(map[string]any)["creation_time"]; hasCreationTime {
				creationTime := strings.ReplaceAll(creationTime.(string), "T", " ")
				creationTime = creationTime[:strings.Index(creationTime, ".")]
				date, _ := time.Parse("2006-01-02 15:04:05", creationTime)
				info.Date = date.UnixMilli()
			}
		}
		if strings.Contains(string(infoData), "\"rotation\": -90") ||
			strings.Contains(string(infoData), "\"rotation\": 90") { //这个视频宽高可能需要调换
			width := info.Width
			info.Width = info.Height
			info.Height = width
		}
		return info, nil
	}
	return VedioInfo{}, exception.Biz("没有获取到视频信息")
}

/**
 * 视频转码
 */
func Transfer(path string, targetW int, targetH int, targetFps float32, targetPath string) error {
	_, errResult, cmdErr := ShellUtil.ExecToOkAndErrorResult(fmt.Sprintf("\"%s/ffmpeg\" -i \"%s\" -vf scale=%d:%d -r %f -f mp4 \"%s\"", application.FfmpegPath, path, targetW, targetH, targetFps, targetPath))
	if cmdErr != nil {
		return exception.Biz(errResult)
	}
	return nil
}
