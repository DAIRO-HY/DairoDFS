package VideoUtil

import (
	application "DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/String"
	"DairoDFS/util/RamDiskUtil"
	"DairoDFS/util/ShellUtil"
	"encoding/json"
	"os"
	"strconv"
	"strings"
	"time"
)

// 生成视频Jpg缩略图
// path 视频文件路径
// targetMaxSize 图片最大边
// return 图片字节数组,错误信息
func ToJpg(path string) ([]byte, error) {
	if IsHDR(path) {
		tempPath := RamDiskUtil.GetRamFolder() + "/" + String.MakeRandStr(16)
		defer os.Remove(tempPath)
		arg := TransferArgument{
			Input:       path,
			Time:        1,
			Fps:         1,
			Crf:         18,
			DeleteSound: true,
			Output:      tempPath,
		}

		//先将HDR转SDR再截图，避免取出来的图片泛白
		if err := HDR2SDR(arg); err != nil {
			return nil, err
		}
		return ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -i \"" + tempPath + "\" -vf select=eq(n\\,0) -q:v 1 -f image2pipe -vcodec mjpeg -")
	} else { //SDR获取截图的方式
		return ShellUtil.ExecToOkData("\"" + application.FfmpegPath + "/ffmpeg\" -i \"" + path + "\" -vf select=eq(n\\,0) -q:v 1 -f image2pipe -vcodec mjpeg -")
	}
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
			if creationTimeAny, hasCreationTime := tags.(map[string]any)["creation_time"]; hasCreationTime {
				creationTime := strings.ReplaceAll(creationTimeAny.(string), "T", " ")
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

// Transfer - SDR视频转SDR视频，HDR视频转HDR视频，
// width,height必须是偶数
func Transfer(arg TransferArgument) error {
	_, errResult, cmdErr := ShellUtil.ExecToOkAndErrorResult(arg.toTransferCmd())
	if cmdErr != nil {
		return exception.Biz(errResult)
	}
	return nil
}

// HDR2SDR - HDR视频转SDR视频
// width,height必须是偶数
func HDR2SDR(arg TransferArgument) error {
	_, errResult, cmdErr := ShellUtil.ExecToOkAndErrorResult(arg.toHDR2SDRCmd())
	if cmdErr != nil {
		return exception.Biz(errResult)
	}
	return nil
}

// IsHDR - 判断视频是否HDR
func IsHDR(path string) bool {
	streamsInfo, errMsg, cmdErr := ShellUtil.ExecToOkAndErrorResult("\"" + application.FfprobePath + "/ffprobe\" -v error -select_streams v:0 -show_entries stream=color_transfer -of json \"" + path + "\"")
	if errMsg != "" || cmdErr != nil {
		return false
	}
	if strings.Contains(streamsInfo, "\"color_transfer\": \"arib-std-b67\"") || strings.Contains(streamsInfo, "\"color_transfer\": \"smpte2084\"") {
		return true
	}
	return false
}

// 转码参数
type TransferArgument struct {

	//输入文件
	Input string

	//裁剪开始时间
	Start int

	//裁剪时长
	Time int

	//目标分辨率：宽(只能是偶数)
	Width int

	//目标分辨率：高(只能是偶数)
	Height int

	//目标帧率
	Fps float32

	//视频画质，0-51 0：无损（文件大） 51：最低画质（文件小）
	Crf int

	// 禁用声音
	DeleteSound bool

	// 输出文件
	Output string
}

// SDR视频转SDR视频，HDR视频转HDR视频，
// width,height必须是偶数
func (mine TransferArgument) toTransferCmd() string {
	cmd := `"${FfmpegPath}/ffmpeg" -i "${input}" -ss ${start} ${time} -vf scale=w=${width}:h=${height} -c:v libx265 -crf ${crf} -preset medium ${fps} -f mp4 ${deleteSound} -y "${output}"`
	cmd = strings.ReplaceAll(cmd, "${FfmpegPath}", application.FfmpegPath)
	cmd = strings.ReplaceAll(cmd, "${input}", mine.Input)
	cmd = strings.ReplaceAll(cmd, "${start}", strconv.Itoa(mine.Start))
	cmd = strings.ReplaceAll(cmd, "${time}", Bool.Is(mine.Time > 0, "-t "+strconv.Itoa(mine.Time), ""))
	cmd = strings.ReplaceAll(cmd, "${width}", strconv.Itoa(mine.Width))
	cmd = strings.ReplaceAll(cmd, "${height}", strconv.Itoa(mine.Height))
	cmd = strings.ReplaceAll(cmd, "${crf}", String.ValueOf(mine.Crf))
	cmd = strings.ReplaceAll(cmd, "${fps}", Bool.Is(mine.Fps > 0, "-r "+strconv.FormatFloat(float64(mine.Fps), 'f', 2, 64), ""))
	cmd = strings.ReplaceAll(cmd, "${deleteSound}", Bool.Is(mine.DeleteSound, "-an", ""))
	cmd = strings.ReplaceAll(cmd, "${output}", mine.Output)
	return cmd
}

// /获取HDR转SDR的指令
func (mine TransferArgument) toHDR2SDRCmd() string {
	cmd := `"${FfmpegPath}/ffmpeg" -i "${input}" -ss ${start} ${time} -vf zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=w=${width}:h=${height}:t=bt709:m=bt709:r=tv,format=yuv420p -c:v libx265 -crf ${crf} -preset medium ${fps} -f mp4 ${deleteSound} -y "${output}"`
	cmd = strings.ReplaceAll(cmd, "${FfmpegPath}", application.FfmpegPath)
	cmd = strings.ReplaceAll(cmd, "${FfmpegPath}", application.FfmpegPath)
	cmd = strings.ReplaceAll(cmd, "${input}", mine.Input)
	cmd = strings.ReplaceAll(cmd, "${start}", strconv.Itoa(mine.Start))
	cmd = strings.ReplaceAll(cmd, "${time}", Bool.Is(mine.Time > 0, "-t "+strconv.Itoa(mine.Time), ""))
	cmd = strings.ReplaceAll(cmd, "${width}", strconv.Itoa(mine.Width))
	cmd = strings.ReplaceAll(cmd, "${height}", strconv.Itoa(mine.Height))
	cmd = strings.ReplaceAll(cmd, "${crf}", String.ValueOf(mine.Crf))
	cmd = strings.ReplaceAll(cmd, "${fps}", Bool.Is(mine.Fps > 0, "-r "+strconv.FormatFloat(float64(mine.Fps), 'f', 2, 64), ""))
	cmd = strings.ReplaceAll(cmd, "${deleteSound}", Bool.Is(mine.DeleteSound, "-an", ""))
	cmd = strings.ReplaceAll(cmd, "${output}", mine.Output)
	return cmd
}
