package DfsFileUtil

import (
	"DairoDFS/appication/SystemConfig"
	_ "embed"
	"fmt"
	"log"
	"testing"
)

// 通过文件名获取文件的content-type
func TestDfsContentType(t *testing.T) {
	contentType := DfsContentType("MP4")
	log.Println(contentType)
}
func TestSelectDriverFolder(t *testing.T) {
	config := SystemConfig.Instance()
	config.UploadMaxSize = 1000 * 1024 * 1024 * 1024
	folder, e := SelectDriverFolder()
	fmt.Println(e)
	fmt.Println(folder)
}
func TestLocalPath(t *testing.T) {
	path, e := LocalPath()
	fmt.Println(e)
	fmt.Println(path)
	path, e = LocalPath()
	fmt.Println(e)
	fmt.Println(path)
}
func TestCheckPath(t *testing.T) {
	if CheckPath("/dsf/sdfsdfs/df/sdf") != nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/sdf.>") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/sdf.?") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/s\\df.txt") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/s:df.txt") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/s|df.txt") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/s*.txt") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("/dsf/df/s\".txt") == nil {
		t.Error("CheckPath处理失败")
	}
	if CheckPath("//dsf/df/s.txt") == nil {
		t.Error("CheckPath处理失败")
	}
}
