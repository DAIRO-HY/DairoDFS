package SyncInfoManager

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/util/SyncUtil"
)

// 当前同步主机信息
var SyncInfoList []*SyncUtil.SyncServerInfo

// 重新加载同步信息
func ReloadList() {
	for _, it := range SyncInfoList {
		it.Cancel() //停止所有同步操作
	}
	SyncInfoList = make([]*SyncUtil.SyncServerInfo, 0)
	for i, it := range SystemConfig.Instance().SyncDomains {
		info := &SyncUtil.SyncServerInfo{
			Url: it,
			No:  i + 1,
			Msg: "等待同步中",
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
