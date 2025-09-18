package DfsFileHandleUtil

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ImageUtil/HeicUtil"
	"DairoDFS/util/ImageUtil/PSDUtil"
	"DairoDFS/util/ImageUtil/RawUtil"
	"DairoDFS/util/VideoUtil"
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"
)

// 标记是否有新的数据
var hasData = false

func HasData() bool {
	return hasData
}

var cond = sync.NewCond(&sync.Mutex{}) // 条件变量

func init() {
	go start()
}

// 发送通知唤醒 Worker
func NotifyWorker() {
	cond.L.Lock()

	//这里设置为true之后，在大量并发的情况下，工作线程有可能恰好将其设置为false，导致数据不能及时处理，只能等到下一次新的数据进来时才能处理，这是超小概率事件，暂不做处理
	hasData = true
	cond.L.Unlock()
	cond.Signal()
}

func start() {
	for {
		cond.L.Lock()
		for !hasData {
			//fmt.Println("文件处理：等待新任务...")
			cond.Wait() // 没数据时进入等待状态
		}
		cond.L.Unlock()

		//fmt.Println("文件处理：开始新任务...")
		for {
			dfsList := DfsFileDao.SelectNoHandle()
			if len(dfsList) == 0 {
				break
			}
			for _, it := range dfsList {
				handle(it)
			}
		}
		cond.L.Lock()
		hasData = false
		cond.L.Unlock()
	}
}

// 处理数据
func handle(it dto.DfsFileDto) {
	defer func() {
		if r := recover(); r != nil { //处理错误
			var errMsg string
			switch rType := r.(type) {
			case string:
				errMsg = rType
			case error:
				errMsg = rType.Error()
			default:
				errMsg = fmt.Sprintf("%q", r)
			}
			DfsFileDao.SetState(it.Id, 2, errMsg)
		}
	}()
	startTime := time.Now().UnixMilli()

	//生成附属文件
	makeExtra(it)

	//耗时
	measureTime := time.Now().UnixMilli() - startTime
	DfsFileDao.SetState(it.Id, 1, "耗时:"+String.ValueOf(int(float64(measureTime)/1000/60))+"分")
}

/**
 * 生成附属文件，如标清视频，高清视频，raw预览图片
 */
func makeExtra(dfsFileDto dto.DfsFileDto) {
	if _, isExistsStorageFile := StorageFileDao.SelectOne(dfsFileDto.StorageId); !isExistsStorageFile {

		//理论上没有不存在的本地文件
		panic(exception.Biz("文件：" + String.ValueOf(dfsFileDto.StorageId) + "不存在"))
	}

	//生成文件属性
	makeProperty(dfsFileDto)

	//获取缩略图
	makeThumb(dfsFileDto)

	// 某些文件生成预览图,如PSD,PDF,RAW等格式的图片
	makePreview(dfsFileDto)

	// 生成视频,如标清视频，高清视频
	makeVideo(dfsFileDto)
}

// 生成文件属性
func makeProperty(dfsFileDto dto.DfsFileDto) {
	if exitsProperty := DfsFileDao.SelectPropertyByStorageId(dfsFileDto.StorageId); exitsProperty != "" {

		//该文件属性已经存在
		DfsFileDao.SetProperty(dfsFileDto.Id, exitsProperty)
		return
	}
	localDto, _ := StorageFileDao.SelectOne(dfsFileDto.StorageId)
	storagePath := localDto.Path
	var property any
	var makePropertyErr error
	ext := dfsFileDto.Ext
	if ext == "jpg" ||
		ext == "jpeg" ||
		ext == "png" ||
		ext == "bmp" ||
		ext == "gif" ||
		ext == "ico" ||
		ext == "svg" ||
		ext == "tiff" ||
		ext == "webp" ||
		ext == "wmf" ||
		ext == "wmz" ||
		ext == "jp2" ||
		ext == "eps" ||
		ext == "tga" ||
		ext == "jfif" { //图片处理
		property, makePropertyErr = ImageUtil.GetInfo(storagePath)
	} else if ext == "psd" || ext == "psb" || ext == "ai" {
		property, makePropertyErr = PSDUtil.GetInfo(storagePath)
	} else if ext == "cr3" || ext == "cr2" { //专业相机RAW图片
		property, makePropertyErr = RawUtil.GetInfo(storagePath)
	} else if ext == "heic" { //Iphone手机拍摄的照片
		property, makePropertyErr = HeicUtil.GetInfo(storagePath)
	} else if ext == "mp4" ||
		ext == "mov" ||
		ext == "avi" ||
		ext == "mkv" ||
		ext == "flv" ||
		ext == "rm" ||
		ext == "rmvb" ||
		ext == "3gp" {
		property, makePropertyErr = VideoUtil.GetInfo(storagePath)
	} else {
		return
	}
	if makePropertyErr != nil {
		panic(makePropertyErr)
	}
	jsonData, _ := json.Marshal(property)
	DfsFileDao.SetProperty(dfsFileDto.Id, string(jsonData))
}

// 生成缩略图
func makeThumb(dfsFileDto dto.DfsFileDto) {

	//获取已经存在的缩略图
	if _, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "thumb"); isExists { //缩略图已经存在,则跳过
		return
	}

	if existsThumb, isExists := DfsFileDao.SelectExtraFileByStorageIdAndName(dfsFileDto.StorageId, "thumb"); isExists {

		//该文件缩略图在其他文件中已经生成
		existsThumb.Id = Number.ID()
		existsThumb.ParentId = dfsFileDto.Id
		existsThumb.UserId = dfsFileDto.UserId
		DfsFileService.Add(existsThumb)
		return
	}

	//去获取预览图
	jpgData, previewErr := GetPreviewJpg(dfsFileDto)
	if previewErr != nil {
		panic(previewErr)
	}
	if jpgData == nil { //无需生成缩略图
		return
	}

	//生成目标图片最大边
	targetMaxSize := SystemConfig.Instance().ThumbMaxSize
	data, makeThumbErr := ImageUtil.ResizeByData(jpgData, targetMaxSize, 85)
	if makeThumbErr != nil {
		panic(makeThumbErr)
	}

	//保存文件
	storageFileDto := DfsFileService.SaveToStorageByData(data)

	//添加缩率图附属文件
	extraDto := dto.DfsFileDto{
		Id:          Number.ID(),
		Name:        "thumb",
		Size:        int64(len(data)),
		StorageId:   storageFileDto.Id,
		IsExtra:     true,
		ParentId:    dfsFileDto.Id,
		UserId:      dfsFileDto.UserId,
		Date:        dfsFileDto.Date,
		State:       1,
		ContentType: DfsFileUtil.DfsContentType("jpeg"),
	}
	DfsFileService.Add(extraDto)
}

// 获取文件预览图片Jpg
func GetPreviewJpg(dfsFileDto dto.DfsFileDto) ([]byte, error) {
	localDto, _ := StorageFileDao.SelectOne(dfsFileDto.StorageId)
	storagePath := localDto.Path

	//缩略图质量
	quality := 100
	ext := dfsFileDto.Ext
	if ext == "bmp" ||
		ext == "gif" ||
		ext == "ico" ||
		ext == "svg" ||
		ext == "webp" ||
		ext == "wmf" ||
		ext == "wmz" ||
		ext == "jp2" ||
		ext == "eps" ||
		ext == "tga" ||
		ext == "jfif" {
		return ImageUtil.ToJpg(storagePath, quality)
	} else if ext == "jpg" ||
		ext == "jpeg" {
		return os.ReadFile(storagePath)
	} else if ext == "png" {
		return ImageUtil.ToJpg(storagePath, quality)
	} else if ext == "tiff" {
		return ImageUtil.ToJpg(storagePath, quality)
	} else if ext == "psd" ||
		ext == "psb" ||
		ext == "ai" {
		return PSDUtil.ToJpg(storagePath)
	} else if ext == "mp4" ||
		ext == "mov" ||
		ext == "avi" ||
		ext == "mkv" ||
		ext == "flv" ||
		ext == "rm" ||
		ext == "rmvb" ||
		ext == "3gp" {
		return VideoUtil.ToJpg(storagePath)
	} else if ext == "cr3" ||
		ext == "cr2" { //专业相机RAW图片
		return RawUtil.ToJpg(storagePath)
	} else if ext == "heic" { //Iphone手机拍摄的照片
		return HeicUtil.ToJpg(storagePath, quality)
	} else { //无需生成缩略图
		return nil, nil
	}
}

// 某些文件生成预览图,如PSD,PDF,RAW等格式的图片
func makePreview(dfsFileDto dto.DfsFileDto) {

	//获取已经存在的附属文件
	if _, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "preview"); isExists {

		//已经存在附属文件,则跳过  重新生成附属文件时用到
		return
	}

	if existsPreview, isExists := DfsFileDao.SelectExtraFileByStorageIdAndName(dfsFileDto.StorageId, "preview"); isExists {

		//该文件预览图在其他文件中已经生成
		existsPreview.Id = Number.ID()
		existsPreview.ParentId = dfsFileDto.Id
		existsPreview.UserId = dfsFileDto.UserId
		DfsFileService.Add(existsPreview)
		return
	}

	var previewData []byte
	var err error
	ext := dfsFileDto.Ext
	if ext == "psd" ||
		ext == "psb" ||
		ext == "ai" ||
		ext == "cr3" ||
		ext == "cr2" ||
		ext == "heic" {
		previewData, err = GetPreviewJpg(dfsFileDto)
	} else {
		return
	}
	if err != nil {
		panic(err)
	}

	//将图片压缩
	previewData, err = ImageUtil.ToJpgByData(previewData, 80)
	if err != nil {
		panic(err)
	}

	//保存文件
	storageFileDto := DfsFileService.SaveToStorageByData(previewData)
	extraDto := dto.DfsFileDto{
		Id:          Number.ID(),
		Name:        "preview",
		Size:        int64(len(previewData)),
		StorageId:   storageFileDto.Id,
		IsExtra:     true,
		ParentId:    dfsFileDto.Id,
		UserId:      dfsFileDto.UserId,
		Date:        dfsFileDto.Date,
		ContentType: "image/jpeg",
		State:       1,
	}
	DfsFileService.Add(extraDto)
}

// 某些文件生成预览图,如PSD,PDF,RAW等格式的图片
func makePreviewBk(dfsFileDto dto.DfsFileDto) {

	//获取已经存在的附属文件
	if _, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "preview"); isExists {

		//已经存在附属文件,则跳过  重新生成附属文件时用到
		return
	}

	if existsPreview, isExists := DfsFileDao.SelectExtraFileByStorageIdAndName(dfsFileDto.StorageId, "preview"); isExists {

		//该文件预览图在其他文件中已经生成
		existsPreview.Id = Number.ID()
		existsPreview.ParentId = dfsFileDto.Id
		existsPreview.UserId = dfsFileDto.UserId
		DfsFileService.Add(existsPreview)
		return
	}
	previewData, previewErr := GetPreviewJpg(dfsFileDto)
	if previewErr != nil {
		panic(previewErr)
	}

	//得到一张低分辨率的预览图，用来生成缩略图等
	previewData, previewErr = ImageUtil.ResizeByData(previewData, 1200, 85)
	if previewErr != nil {
		panic(previewErr)
	}

	//保存文件
	storageFileDto := DfsFileService.SaveToStorageByData(previewData)
	extraDto := dto.DfsFileDto{
		Id:          Number.ID(),
		Name:        "preview",
		Size:        int64(len(previewData)),
		StorageId:   storageFileDto.Id,
		IsExtra:     true,
		ParentId:    dfsFileDto.Id,
		UserId:      dfsFileDto.UserId,
		Date:        dfsFileDto.Date,
		ContentType: "image/jpeg",
		State:       1,
	}
	DfsFileService.Add(extraDto)
}

// 生成视频,如标清视频，高清视频
func makeVideo(dfsFileDto dto.DfsFileDto) {
	localDto, _ := StorageFileDao.SelectOne(dfsFileDto.StorageId)
	storagePath := localDto.Path
	ext := dfsFileDto.Ext
	if ext == "mp4" ||
		ext == "mov" ||
		ext == "avi" ||
		ext == "mkv" ||
		ext == "flv" ||
		ext == "rm" ||
		ext == "rmvb" ||
		ext == "3gp" {

		videoInfo, err := VideoUtil.GetInfo(storagePath)
		if err != nil {
			panic(err)
		}

		//要转换的目标尺寸,TODO:应该从配置文件里获取
		targetEncode := "1280:30"
		targetArr := strings.Split(targetEncode, ":")
		targetSizeInt64, _ := strconv.ParseInt(targetArr[0], 10, 16) //目标最大边
		targetFpsInt64, _ := strconv.ParseInt(targetArr[1], 10, 16)  //目标帧数

		targetSize := int(targetSizeInt64)
		targetFps := float32(targetFpsInt64)

		//获取已经存在的附属文件
		if _, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "preview"); isExists {

			//已经存在附属文件,则跳过  重新生成附属文件时用到
			return
		}

		//同样的文件，是否在其他地方已经生成了预览视频
		if existsVideo, isExists := DfsFileDao.SelectExtraFileByStorageIdAndName(dfsFileDto.StorageId, "preview"); isExists {

			//该文件预览图在其他文件中已经生成
			existsVideo.Id = Number.ID()
			existsVideo.ParentId = dfsFileDto.Id
			existsVideo.UserId = dfsFileDto.UserId
			DfsFileService.Add(existsVideo)
			return
		}

		//是否横向视频
		isHorizontal := videoInfo.Width > videoInfo.Height

		//视频文件最最大边像素
		maxSize := Bool.Is(isHorizontal, videoInfo.Width, videoInfo.Height)
		if targetSize > maxSize { //视频最大边小于当前要转换的目标尺寸，则跳过
			return
		}

		//当视频宽度相等时,如果目标视频帧数大于或者等于原视频帧数,则不需要处理
		if targetSize == maxSize && targetFps >= videoInfo.Fps {
			return
		}
		if targetFps > videoInfo.Fps {
			targetFps = videoInfo.Fps
		}

		var targetW int   //目标宽
		var targetH int   //目标高
		if isHorizontal { //如果是横向视频
			targetW = targetSize
			targetH = int(math.Round(float64(targetW) / float64(videoInfo.Width) * float64(videoInfo.Height)))
			if targetH%2 == 1 { //视频像素不能时基数
				targetH -= 1
			}
		} else { //如果是竖向视频
			targetH = targetSize
			targetW = int(math.Round(float64(targetH) * float64(videoInfo.Width) / float64(videoInfo.Height)))
			if targetW%2 == 1 { //视频像素不能时基数
				targetW -= 1
			}
		}

		//转换之后的文件
		targetPathRelative := application.DataPath + "/temp/" + String.ValueOf(time.Now().UnixMicro())
		targetPath, _ := filepath.Abs(targetPathRelative)
		arg := VideoUtil.TransferArgument{
			Input:  storagePath,
			Width:  targetW,
			Height: targetH,
			Fps:    targetFps,
			Crf:    22,
			Output: targetPath,
		}

		//删除转码的文件
		defer os.Remove(targetPath)

		//将视频转码成SDR高兼容性
		if VideoUtil.IsHDR(storagePath) {
			if transferErr := VideoUtil.HDR2SDR(arg); transferErr != nil {
				panic(transferErr)
			}
		} else {
			if transferErr := VideoUtil.Transfer(arg); transferErr != nil {
				panic(transferErr)
			}
		}

		//保存到本地文件
		storageFileDto := DfsFileService.SaveToStorageByFile(targetPath, "")
		targetFileInfo, _ := os.Stat(targetPath)
		extraDto := dto.DfsFileDto{
			Id:          Number.ID(),
			Name:        "preview",
			Size:        targetFileInfo.Size(),
			StorageId:   storageFileDto.Id,
			IsExtra:     true,
			ParentId:    dfsFileDto.Id,
			UserId:      dfsFileDto.UserId,
			Date:        dfsFileDto.Date,
			ContentType: "video/mp4",
			State:       1,
		}
		DfsFileService.Add(extraDto)
	}
}
