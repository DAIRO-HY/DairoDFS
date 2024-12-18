package DfsFileUtil

import (
	_ "embed"
	"fmt"
	"github.com/shirou/gopsutil/disk"
	"log"
	"testing"
)

// 通过文件名获取文件的content-type
func TestDfsContentType(t *testing.T) {
	contentType := dfsContentType("MP4")
	log.Println(contentType)
}

func getDiskFreeSpace(path string) (uint64, error) {
	usage, err := disk.Usage(path)
	if err != nil {
		return 0, err
	}
	return usage.Free, nil
}
func TestSelectDriverFolder(t *testing.T) {
	s, e := getDiskFreeSpace("C:\\develop\\project\\idea\\DairoDFS\\util")
	if e != nil {
		log.Fatal(e)
	}
	fmt.Println(s)
}
