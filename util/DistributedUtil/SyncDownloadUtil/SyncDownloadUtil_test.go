package SyncDownloadUtil

import (
	"DairoDFS/application"
	"DairoDFS/util/DistributedUtil"
	"fmt"
	"testing"
)

func TestDownload(t *testing.T) {
	application.Init()
	info := &DistributedUtil.SyncServerInfo{
		Url: "http://localhost:8031/index.html?",
		//Url: "http://home.dfs.jp.dairo.cn/d/s7LlmR/%E8%BD%AF%E4%BB%B6/openvpn-connect-3.5.0.3818_signed.msi?",
	}
	path, err := Download(info, "index.html", 0)
	if err != nil {
		t.Fatal(err)
	}
	fmt.Println(path)
}
