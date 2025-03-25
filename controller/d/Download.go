package distributed

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/UserDao"
	"DairoDFS/util/DfsFileUtil"
	"net/http"
)

// 文件下载
// @Get:/d/{urlPath}/{path}
func Download(writer http.ResponseWriter, request *http.Request, urlPath string, path string) {
	userId := UserDao.SelectIdByUrlPath(urlPath)
	if userId == 0 { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	fileId := DfsFileDao.SelectIdByPath(userId, "/"+path)
	if fileId == 0 { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	DfsFileUtil.DownloadDfsId(fileId, writer, request)
}
