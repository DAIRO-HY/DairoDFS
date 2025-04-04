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
	if dfsFileDto.Ext == ".jpg" ||
		dfsFileDto.Ext == "jpeg" ||
		dfsFileDto.Ext == "png" ||
		dfsFileDto.Ext == "bmp" ||
		dfsFileDto.Ext == "gif" ||
		dfsFileDto.Ext == "ico" ||
		dfsFileDto.Ext == "svg" ||
		dfsFileDto.Ext == "tiff" ||
		dfsFileDto.Ext == "webp" ||
		dfsFileDto.Ext == "wmf" ||
		dfsFileDto.Ext == "wmz" ||
		dfsFileDto.Ext == "jp2" ||
		dfsFileDto.Ext == "eps" ||
		dfsFileDto.Ext == "tga" ||
		dfsFileDto.Ext == "jfif" { //图片处理
		property, makePropertyErr = ImageUtil.GetInfo(storagePath)
	} else if dfsFileDto.Ext == "psd" || dfsFileDto.Ext == "psb" || dfsFileDto.Ext == "ai" {
		property, makePropertyErr = PSDUtil.GetInfo(storagePath)
	} else if dfsFileDto.Ext == "cr3" || dfsFileDto.Ext == "cr2" { //专业相机RAW图片
		property, makePropertyErr = RawUtil.GetInfo(storagePath)
	} else if dfsFileDto.Ext == "heic" { //Iphone手机拍摄的照片
		property, makePropertyErr = HeicUtil.GetInfo(storagePath)
	} else if dfsFileDto.Ext == "mp4" ||
		dfsFileDto.Ext == "mov" ||
		dfsFileDto.Ext == "avi" ||
		dfsFileDto.Ext == "mkv" ||
		dfsFileDto.Ext == "flv" ||
		dfsFileDto.Ext == "rm" ||
		dfsFileDto.Ext == "rmvb" ||
		dfsFileDto.Ext == "3gp" {
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

	localDto, _ := StorageFileDao.SelectOne(dfsFileDto.StorageId)
	storagePath := localDto.Path

	//缩略图数据
	var data []byte

	//生成缩略图过程中出现的错误
	var makeThumbErr error
	lowerName := strings.ToLower(dfsFileDto.Name)

	//生成目标图片最大边
	targetMaxSize := SystemConfig.Instance().ThumbMaxSize

	//默认文件类型
	contentType := DfsFileUtil.DfsContentType("jpeg")
	if strings.HasSuffix(lowerName, ".bmp") ||
		strings.HasSuffix(lowerName, ".gif") ||
		strings.HasSuffix(lowerName, ".ico") ||
		strings.HasSuffix(lowerName, ".svg") ||
		strings.HasSuffix(lowerName, ".webp") ||
		strings.HasSuffix(lowerName, ".wmf") ||
		strings.HasSuffix(lowerName, ".wmz") ||
		strings.HasSuffix(lowerName, ".jp2") ||
		strings.HasSuffix(lowerName, ".eps") ||
		strings.HasSuffix(lowerName, ".tga") ||
		strings.HasSuffix(lowerName, ".jfif") {
		data, makeThumbErr = ImageUtil.ThumbByFile(storagePath, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".jpg") ||
		strings.HasSuffix(lowerName, ".jpeg") {
		jpgData, _ := os.ReadFile(storagePath)
		data, makeThumbErr = ImageUtil.ThumbByJpg(jpgData, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".png") {
		pngData, _ := os.ReadFile(storagePath)
		data, makeThumbErr = ImageUtil.ThumbByPng(pngData, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".tiff") {
		data, makeThumbErr = ImageUtil.ThumbByTiffPath(storagePath, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".psd") ||
		strings.HasSuffix(lowerName, ".psb") ||
		strings.HasSuffix(lowerName, ".ai") {
		data, makeThumbErr = PSDUtil.Thumb(storagePath, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".mp4") ||
		strings.HasSuffix(lowerName, ".avi") ||
		strings.HasSuffix(lowerName, ".mkv") ||
		strings.HasSuffix(lowerName, ".flv") ||
		strings.HasSuffix(lowerName, ".rm") ||
		strings.HasSuffix(lowerName, ".rmvb") ||
		strings.HasSuffix(lowerName, ".3gp") {
		data, makeThumbErr = VideoUtil.Thumb(storagePath, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".mov") {
		data, makeThumbErr = VideoUtil.ThumbPng(storagePath, targetMaxSize)
		contentType = DfsFileUtil.DfsContentType("png")
	} else if strings.HasSuffix(lowerName, ".cr3") ||
		strings.HasSuffix(lowerName, ".cr2") {

		//专业相机RAW图片
		data, makeThumbErr = RawUtil.Thumb(storagePath, targetMaxSize)
	} else if strings.HasSuffix(lowerName, ".heic") { //Iphone手机拍摄的照片
		data, makeThumbErr = HeicUtil.Thumb(storagePath, targetMaxSize)
	} else { //无需生成缩略图
		return
	}
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
		ContentType: contentType,
	}
	DfsFileService.Add(extraDto)
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

	localDto, _ := StorageFileDao.SelectOne(dfsFileDto.StorageId)
	storagePath := localDto.Path
	lowerName := strings.ToLower(dfsFileDto.Name)

	var previewData []byte
	var err error
	if strings.HasSuffix(lowerName, ".psd") ||
		strings.HasSuffix(lowerName, ".psb") ||
		strings.HasSuffix(lowerName, ".ai") {
		previewData, err = PSDUtil.ToJpeg(storagePath)
	} else if strings.HasSuffix(lowerName, ".cr3") || strings.HasSuffix(lowerName, ".cr2") {
		previewData, err = RawUtil.ToJpg(storagePath)
	} else if strings.HasSuffix(lowerName, ".heic") {
		previewData, err = HeicUtil.ToJpeg(storagePath, 2)
	} else {
		return
	}
	if err != nil {
		return
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
	if dfsFileDto.Ext == "mp4" ||
		dfsFileDto.Ext == "mov" ||
		dfsFileDto.Ext == "avi" ||
		dfsFileDto.Ext == "mkv" ||
		dfsFileDto.Ext == "flv" ||
		dfsFileDto.Ext == "rm" ||
		dfsFileDto.Ext == "rmvb" ||
		dfsFileDto.Ext == "3gp" {

		videoInfo, err := VideoUtil.GetInfo(storagePath)
		if err != nil {
			panic(err)
		}

		//要转换的目标尺寸
		for _, it := range []string{"1920:30", "1280:25", "640:15"} {
			targetArr := strings.Split(it, ":")
			targetSizeInt64, _ := strconv.ParseInt(targetArr[0], 10, 16) //目标最大边
			targetFpsInt64, _ := strconv.ParseInt(targetArr[1], 10, 16)  //目标帧数

			targetSize := int(targetSizeInt64)
			targetFps := float32(targetFpsInt64)

			//获取已经存在的附属文件
			if _, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, String.ValueOf(targetSize)); isExists {

				//已经存在附属文件,则跳过  重新生成附属文件时用到
				continue
			}
			if existsVideo, isExists := DfsFileDao.SelectExtraFileByStorageIdAndName(dfsFileDto.StorageId, String.ValueOf(targetSize)); isExists {

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
				continue
			}

			//当视频宽度相等时,如果目标视频帧数大于或者等于原视频帧数,则不需要处理
			if targetSize == maxSize && targetFps >= videoInfo.Fps {
				continue
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
			if err := VideoUtil.Transfer(storagePath, targetW, targetH, targetFps, targetPath); err != nil {
				panic(err)
			}

			targetFileInfo, _ := os.Stat(targetPath)
			targetFile, _ := os.Open(targetPath)

			//保存到本地文件
			storageFileDto := DfsFileService.SaveToStorageByFile(targetPath, "")
			_ = targetFile.Close()
			_ = os.Remove(targetPath)

			extraDto := dto.DfsFileDto{
				Id:          Number.ID(),
				Name:        String.ValueOf(targetSize),
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
}
