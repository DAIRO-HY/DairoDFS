package DfsFileHandleUtil

import (
	"DairoDFS/application"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/LocalFileDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/File"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/ImageUtil"
	"DairoDFS/util/ImageUtil/PSDUtil"
	"DairoDFS/util/ImageUtil/RawUtil"
	"DairoDFS/util/VideoUtil"
	"bytes"
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
			fmt.Println("Worker 等待新任务...")
			cond.Wait() // 没数据时进入等待状态
		}
		cond.L.Unlock()

		fmt.Println("Worker 开始新任务...")
		for {
			dfsList := DfsFileDao.SelectNoHandle()
			if len(dfsList) == 0 {
				break
			}
			for _, it := range dfsList {
				startTime := time.Now().UnixMilli()

				//设置文件属性
				if err := makeProperty(it); err != nil {
					DfsFileDao.SetState(it.Id, 2, err.Error())
					continue
				}

				//生成附属文件，如标清视频，高清视频，raw预览图片
				if err := makeExtra(it); err != nil {
					DfsFileDao.SetState(it.Id, 2, err.Error())
					continue
				}

				//耗时
				measureTime := time.Now().UnixMilli() - startTime
				DfsFileDao.SetState(it.Id, 1, "耗时:"+String.ValueOf(int(float64(measureTime)/1000/60))+"分")
			}
		}
		cond.L.Lock()
		hasData = false
		cond.L.Unlock()
	}
}

/**
 * 生成文件属性
 */
func makeProperty(dfsFileDto dto.DfsFileDto) error {
	exitsProperty := DfsFileDao.SelectPropertyByLocalId(dfsFileDto.LocalId)
	if exitsProperty != "" { //属性已经存在
		DfsFileDao.SetProperty(dfsFileDto.Id, exitsProperty)
		return nil
	}
	localDto, isExists := LocalFileDao.SelectOne(dfsFileDto.LocalId)
	if !isExists { //理论上没有不存在的文件
		return nil
	}
	path := localDto.Path
	lowerName := strings.ToLower(dfsFileDto.Name)
	var property any
	var makePropertyErr error
	if strings.HasSuffix(lowerName, ".jpg") ||
		strings.HasSuffix(lowerName, ".jpeg") ||
		strings.HasSuffix(lowerName, ".png") ||
		strings.HasSuffix(lowerName, ".bmp") ||
		strings.HasSuffix(lowerName, ".gif") ||
		strings.HasSuffix(lowerName, ".ico") ||
		strings.HasSuffix(lowerName, ".svg") ||
		strings.HasSuffix(lowerName, ".tiff") ||
		strings.HasSuffix(lowerName, ".webp") ||
		strings.HasSuffix(lowerName, ".wmf") ||
		strings.HasSuffix(lowerName, ".wmz") ||
		strings.HasSuffix(lowerName, ".jp2") ||
		strings.HasSuffix(lowerName, ".eps") ||
		strings.HasSuffix(lowerName, ".tga") ||
		strings.HasSuffix(lowerName, ".jfif") { //图片处理
		property, makePropertyErr = ImageUtil.GetInfo(path)
	} else if strings.HasSuffix(lowerName, ".psd") ||
		strings.HasSuffix(lowerName, ".psb") ||
		strings.HasSuffix(lowerName, ".ai") {
		property, makePropertyErr = PSDUtil.GetInfo(path)
	} else if strings.HasSuffix(lowerName, ".mp4") ||
		strings.HasSuffix(lowerName, ".mov") ||
		strings.HasSuffix(lowerName, ".avi") ||
		strings.HasSuffix(lowerName, ".mkv") ||
		strings.HasSuffix(lowerName, ".flv") ||
		strings.HasSuffix(lowerName, ".rm") ||
		strings.HasSuffix(lowerName, ".rmvb") ||
		strings.HasSuffix(lowerName, ".3gp") {
		property, makePropertyErr = VideoUtil.GetInfo(path)
	} else if strings.HasSuffix(lowerName, ".cr3") || strings.HasSuffix(lowerName, ".cr2") { //专业相机RAW图片
		property, makePropertyErr = RawUtil.GetInfo(path)
	} else {
	}
	if makePropertyErr != nil {
		return makePropertyErr
	}
	jsonData, _ := json.Marshal(property)
	DfsFileDao.SetProperty(dfsFileDto.Id, string(jsonData))
	return nil
}

/**
 * 生成附属文件，如标清视频，高清视频，raw预览图片
 */
func makeExtra(dfsFileDto dto.DfsFileDto) error {
	existsExtraList := DfsFileDao.SelectExtraFileByLocalId(dfsFileDto.LocalId)
	if len(existsExtraList) > 0 { //该文件已经存在了附属文件,直接使用
		for _, it := range existsExtraList {
			extraDto := dto.DfsFileDto{
				Id:          DBUtil.ID(),
				Name:        it.Name,
				Size:        it.Size,
				LocalId:     it.LocalId,
				IsExtra:     true,
				ParentId:    dfsFileDto.Id,
				UserId:      it.UserId,
				Date:        dfsFileDto.Date,
				State:       1,
				ContentType: it.ContentType,
			}
			DfsFileDao.Add(extraDto)
		}
		return nil
	}

	//获取缩略图
	makeThumbErr := makeThumb(dfsFileDto)
	if makeThumbErr != nil {
		return makeThumbErr
	}
	localDto, isExistsLocalFile := LocalFileDao.SelectOne(dfsFileDto.LocalId)
	if !isExistsLocalFile { //理论上没有不存在的本地文件
		return exception.Biz("文件：" + String.ValueOf(dfsFileDto.LocalId) + "不存在")
	}
	path := localDto.Path
	lowerName := strings.ToLower(dfsFileDto.Name)
	//if strings.HasSuffix(lowerName, ".jpg") ||
	//	strings.HasSuffix(lowerName, ".jpeg") ||
	//	strings.HasSuffix(lowerName, ".png") ||
	//	strings.HasSuffix(lowerName, ".bmp") ||
	//	strings.HasSuffix(lowerName, ".gif") ||
	//	strings.HasSuffix(lowerName, ".ico") ||
	//	strings.HasSuffix(lowerName, ".svg") ||
	//	strings.HasSuffix(lowerName, ".tiff") ||
	//	strings.HasSuffix(lowerName, ".webp") ||
	//	strings.HasSuffix(lowerName, ".wmf") ||
	//	strings.HasSuffix(lowerName, ".wmz") ||
	//	strings.HasSuffix(lowerName, ".jp2") ||
	//	strings.HasSuffix(lowerName, ".eps") ||
	//	strings.HasSuffix(lowerName, ".tga") ||
	//	strings.HasSuffix(lowerName, ".jfif") {
	//
	//} else
	if strings.HasSuffix(lowerName, ".psd") ||
		strings.HasSuffix(lowerName, ".psb") ||
		strings.HasSuffix(lowerName, ".ai") {

		//获取已经存在的附属文件
		_, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "preview")
		if isExists { //已经存在附属文件,则跳过  重新生成附属文件时用到
			return nil
		}
		pngData, err := PSDUtil.ToPng(path)
		if err != nil {
			return err
		}
		md5 := File.ToMd5ByBytes(pngData)

		//保存文件
		localFileDto, err := DfsFileService.SaveToLocalFile(md5, bytes.NewReader(pngData))
		if err != nil {
			return err
		}
		extraDto := dto.DfsFileDto{
			Id:          DBUtil.ID(),
			Name:        "preview",
			Size:        int64(len(pngData)),
			LocalId:     localFileDto.Id,
			IsExtra:     true,
			ParentId:    dfsFileDto.Id,
			UserId:      dfsFileDto.UserId,
			Date:        dfsFileDto.Date,
			State:       1,
			ContentType: "image/png",
		}
		DfsFileDao.Add(extraDto)
	} else if strings.HasSuffix(lowerName, ".mp4") ||
		strings.HasSuffix(lowerName, ".mov") ||
		strings.HasSuffix(lowerName, ".avi") ||
		strings.HasSuffix(lowerName, ".mkv") ||
		strings.HasSuffix(lowerName, ".flv") ||
		strings.HasSuffix(lowerName, ".rm") ||
		strings.HasSuffix(lowerName, ".rmvb") ||
		strings.HasSuffix(lowerName, ".3gp") {
		videoInfo, err := VideoUtil.GetInfo(path)
		if err != nil {
			return err
		}

		//要转换的目标尺寸
		for _, it := range []string{"1920:30", "1280:25", "640:15"} {
			targetArr := strings.Split(it, ":")
			targetSizeInt64, _ := strconv.ParseInt(targetArr[0], 10, 16) //目标最大边
			targetFpsInt64, _ := strconv.ParseInt(targetArr[1], 10, 16)  //目标帧数

			targetSize := int(targetSizeInt64)
			targetFps := float32(targetFpsInt64)

			//获取已经存在的附属文件
			_, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, String.ValueOf(targetSize))
			if isExists { //已经存在附属文件,则跳过  重新生成附属文件时用到
				continue
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
			VideoUtil.Transfer(path, targetW, targetH, targetFps, targetPath)
			md5 := File.ToMd5(targetPath)

			targetFileInfo, _ := os.Stat(targetPath)
			targetFile, _ := os.Open(targetPath)

			//保存到本地文件
			localFileDto, saveFileErr := DfsFileService.SaveToLocalFile(md5, targetFile)
			targetFile.Close()
			os.Remove(targetPath)
			if saveFileErr != nil {
				return err
			}

			extraDto := dto.DfsFileDto{
				Id:          DBUtil.ID(),
				Name:        String.ValueOf(targetSize),
				Size:        targetFileInfo.Size(),
				LocalId:     localFileDto.Id,
				IsExtra:     true,
				ParentId:    dfsFileDto.Id,
				UserId:      dfsFileDto.UserId,
				Date:        dfsFileDto.Date,
				ContentType: "video/mp4",
				State:       1,
			}
			DfsFileDao.Add(extraDto)
		}
	} else if strings.HasSuffix(lowerName, ".cr3") || strings.HasSuffix(lowerName, ".cr2") {

		//获取已经存在的附属文件
		_, isExists := DfsFileDao.SelectExtra(dfsFileDto.Id, "preview")
		if isExists { //已经存在附属文件,则跳过  重新生成附属文件时用到
			return nil
		}
		jpgData, err := RawUtil.ToJpg(path)
		if err != nil {
			return err
		}
		md5 := File.ToMd5ByBytes(jpgData)

		//保存文件
		localFileDto, err := DfsFileService.SaveToLocalFile(md5, bytes.NewReader(jpgData))
		if err != nil {
			return err
		}
		extraDto := dto.DfsFileDto{
			Id:          DBUtil.ID(),
			Name:        "preview",
			Size:        int64(len(jpgData)),
			LocalId:     localFileDto.Id,
			IsExtra:     true,
			ParentId:    dfsFileDto.Id,
			UserId:      dfsFileDto.UserId,
			Date:        dfsFileDto.Date,
			ContentType: "image/jpeg",
			State:       1,
		}
		DfsFileDao.Add(extraDto)
	} else {
	}
	return nil
}

/**
 * 生成缩略图
 */
func makeThumb(dfsFileDto dto.DfsFileDto) error {
	localDto, isExists := LocalFileDao.SelectOne(dfsFileDto.LocalId)
	if !isExists {
		return exception.Biz("本地文件ID:" + String.ValueOf(dfsFileDto.LocalId) + "不存在")
	}
	path := localDto.Path

	//缩略图数据
	var data []byte

	//生成缩略图过程中出现的错误
	var makeThumbErr error
	lowerName := strings.ToLower(dfsFileDto.Name)
	width := 300
	height := 300
	if strings.HasSuffix(lowerName, ".jpg") ||
		strings.HasSuffix(lowerName, ".jpeg") ||
		strings.HasSuffix(lowerName, ".png") ||
		strings.HasSuffix(lowerName, ".bmp") ||
		strings.HasSuffix(lowerName, ".gif") ||
		strings.HasSuffix(lowerName, ".ico") ||
		strings.HasSuffix(lowerName, ".svg") ||
		strings.HasSuffix(lowerName, ".tiff") ||
		strings.HasSuffix(lowerName, ".webp") ||
		strings.HasSuffix(lowerName, ".wmf") ||
		strings.HasSuffix(lowerName, ".wmz") ||
		strings.HasSuffix(lowerName, ".jp2") ||
		strings.HasSuffix(lowerName, ".eps") ||
		strings.HasSuffix(lowerName, ".tga") ||
		strings.HasSuffix(lowerName, ".jfif") {
		data, makeThumbErr = ImageUtil.ThumbByFile(path, width, height)
	} else if strings.HasSuffix(lowerName, ".psd") ||
		strings.HasSuffix(lowerName, ".psb") ||
		strings.HasSuffix(lowerName, ".ai") {
		data, makeThumbErr = PSDUtil.Thumb(path, width, height)
	} else if strings.HasSuffix(lowerName, ".mp4") ||
		strings.HasSuffix(lowerName, ".mov") ||
		strings.HasSuffix(lowerName, ".avi") ||
		strings.HasSuffix(lowerName, ".mkv") ||
		strings.HasSuffix(lowerName, ".flv") ||
		strings.HasSuffix(lowerName, ".rm") ||
		strings.HasSuffix(lowerName, ".rmvb") ||
		strings.HasSuffix(lowerName, ".3gp") {
		data, makeThumbErr = VideoUtil.Thumb(path, width, height)
	} else if strings.HasSuffix(lowerName, ".cr3") ||
		strings.HasSuffix(lowerName, ".cr2") {

		//专业相机RAW图片
		data, makeThumbErr = RawUtil.Thumb(path, width, height)
	} else {
	}
	if makeThumbErr != nil {
		return makeThumbErr
	}

	//计算缩略图的md5
	md5 := File.ToMd5ByBytes(data)

	//保存文件
	localFileDto, saveErr := DfsFileService.SaveToLocalFile(md5, bytes.NewReader(data))
	if saveErr != nil {
		return saveErr
	}

	//添加缩率图附属文件
	extraDto := dto.DfsFileDto{
		Id:          DBUtil.ID(),
		Name:        "thumb",
		Size:        int64(len(data)),
		LocalId:     localFileDto.Id,
		IsExtra:     true,
		ParentId:    dfsFileDto.Id,
		UserId:      dfsFileDto.UserId,
		Date:        dfsFileDto.Date,
		State:       1,
		ContentType: DfsFileUtil.DfsContentType("jpeg"),
	}
	return DfsFileDao.Add(extraDto)
}
