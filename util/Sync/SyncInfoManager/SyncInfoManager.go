package SyncInfoManager

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/util/Sync/bean"
	"time"
)

// 当前同步主机信息
var SyncInfoList []*bean.SyncServerInfo

// 重新加载同步信息
func ReloadList() {
	for _, it := range SyncInfoList {
		it.Cancel() //停止所有同步操作
	}
	SyncInfoList = make([]*bean.SyncServerInfo, 0)
	for i, it := range SystemConfig.Instance().SyncDomains {
		info := &bean.SyncServerInfo{
			Url:      it,
			No:       i + 1,
			TestTime: time.Now().UnixMicro(),
			Msg:      "等待同步中",
		}
		SyncInfoList = append(SyncInfoList, info)
	}
}

// 是否有处理失败的数据
func HasError() bool {
	for _, it := range SyncInfoList {
		if it.State == 2 {
			return true
		}
	}
	return false
}

// 停止所有同步操作
func CancelAll() {
	for _, it := range SyncInfoList {
		it.Cancel() //停止之前所有的监听
	}
}
