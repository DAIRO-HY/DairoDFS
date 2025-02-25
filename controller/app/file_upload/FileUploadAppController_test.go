package file_upload

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"testing"
)

func TestByStream(t *testing.T) {
	file, _ := os.Open("C:\\develop\\project\\idea\\DairoDFS-JAVA\\dairo-dfs-server\\src\\main\\resources\\static\\app\\res\\common\\js\\common.js")
	resp, _ := http.Post("http://127.0.0.1:8031/app/file_upload/by_stream/048cd62442dc9084983ad4a8a6c52f17", "", file)
	data, _ := io.ReadAll(resp.Body)
	fmt.Println(string(data))
}

func TestGetUploadedSize(t *testing.T) {

}
