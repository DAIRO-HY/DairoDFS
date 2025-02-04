package SyncFileUtil

import (
	"DairoDFS/application"
	"DairoDFS/util/Sync/bean"
	"testing"
)

func TestDownload(t *testing.T) {
	application.Init()
	info := bean.SyncServerInfo{
		Url: "http://home.dfs.jp.dairo.cn/d/s7LlmR/%E8%BD%AF%E4%BB%B6/openvpn-connect-3.5.0.3818_signed.msi?",
	}
	Download(info, "sfdgdsgdfgerfsddfsfsfsfsdfsf", 0)
}
