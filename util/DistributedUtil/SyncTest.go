package DistributedUtil

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"DairoDFS/extension/String"
	"encoding/json"
	"os"
)

// 仅仅测试用
// 测试时获取主机信息
func GetMasterInfo() *SyncServerInfo {
	systemConfig := &SystemConfig.SystemConfig{}
	data, _ := os.ReadFile("C:\\develop\\project\\idea\\DairoDFS\\data\\system.json")
	json.Unmarshal(data, systemConfig)
	return &SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port) + "/distributed/" + systemConfig.DistributedToken + "/" + SystemConfig.Instance().DistributedToken,
	}
}
