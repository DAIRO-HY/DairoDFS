package Number

import (
	"fmt"
	"sync"
	"time"
)

// 数据流量单位换算
func ToDataSize(input any) string {
	var inputFloat64 float64
	switch v := input.(type) {
	case int:
		inputFloat64 = float64(v)
	case int8:
		inputFloat64 = float64(v)
	case int16:
		inputFloat64 = float64(v)
	case int32:
		inputFloat64 = float64(v)
	case int64:
		inputFloat64 = float64(v)
	case uint:
		inputFloat64 = float64(v)
	case uint8:
		inputFloat64 = float64(v)
	case uint16:
		inputFloat64 = float64(v)
	case uint32:
		inputFloat64 = float64(v)
	case uint64:
		inputFloat64 = float64(v)
	case float32:
		inputFloat64 = float64(v)
	case float64:
		inputFloat64 = v
	default:
		inputFloat64 = 0.0
	}

	var dataSize float64
	var unit string
	if inputFloat64 >= 1024*1024*1024*1024 {
		dataSize = inputFloat64 / (1024 * 1024 * 1024 * 1024)
		unit = "TB"
	} else if inputFloat64 >= 1024*1024*1024 {
		dataSize = inputFloat64 / (1024 * 1024 * 1024)
		unit = "GB"
	} else if inputFloat64 >= 1024*1024 {
		dataSize = inputFloat64 / (1024 * 1024)
		unit = "MB"
	} else if inputFloat64 >= 1024 {
		dataSize = inputFloat64 / 1024
		unit = "KB"
	} else {
		dataSize = inputFloat64
		unit = "B"
	}
	dataSizeStr := fmt.Sprintf("%.2f", dataSize)
	return dataSizeStr + unit
}

// 转换成时间格式
func ToTimeFormat(input any) string {

	var senconds int64
	switch v := input.(type) {
	case int:
		senconds = int64(v)
	case int8:
		senconds = int64(v)
	case int16:
		senconds = int64(v)
	case int32:
		senconds = int64(v)
	case int64:
		senconds = v
	case uint:
		senconds = int64(v)
	case uint8:
		senconds = int64(v)
	case uint16:
		senconds = int64(v)
	case uint32:
		senconds = int64(v)
	case uint64:
		senconds = int64(v)
	case float32:
		senconds = int64(v)
	case float64:
		senconds = int64(v)
	default:
		senconds = 0.0
	}

	//小时
	h := fmt.Sprintf("%02d", senconds/(60*60))

	//分
	m := fmt.Sprintf("%02d", senconds%(60*60)/60)

	//秒
	s := fmt.Sprintf("%02d", senconds%60)
	if senconds >= 60*60 {
		return h + ":" + m + ":" + s
	}
	if senconds >= 60 {
		return m + ":" + s
	}
	return "00:" + s
}

// 生成ID的锁
var makeIdLock sync.Mutex

// 记录上一次生成的id
var preId int64

// ID 生成数据库主键ID
func ID() int64 {
	makeIdLock.Lock()
	defer makeIdLock.Unlock()
	for {
		id := time.Now().UnixMilli()
		if id == preId { //与上次生成的id重复,等待一段时间再生成
			time.Sleep(500 * time.Microsecond)
			continue
		}
		preId = id
		return id
	}
}
