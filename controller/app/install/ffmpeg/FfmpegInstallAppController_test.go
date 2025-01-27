package ffmpeg

import (
	"testing"
)

// 下载测试
func TestDownloadFile(t *testing.T) {
	err := downloadFile(url())
	if err != nil {
		t.Fatal(err)
	}
}
