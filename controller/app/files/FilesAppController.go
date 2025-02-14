package files

import (
	"DairoDFS/controller/app/files/form"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/service/FileShareService"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/LoginState"
	"net/http"
	"strings"
)

// 文件列表页面
//@Group:/app/files

// @Html:.html
func Html() {}

// 获取文件列表
// @Post:/get_list
func GetList(folder string) []form.FileForm {
	loginId := LoginState.LoginId()
	folderId, err := DfsFileService.GetIdByFolder(loginId, folder, false)
	if err != nil {
		return []form.FileForm{}
	}
	list := DfsFileDao.SelectSubFile(loginId, folderId)

	forms := make([]form.FileForm, 0)
	for _, it := range list {
		outForm := form.FileForm{
			Id:       it.Id,
			Name:     it.Name,
			Size:     it.Size,
			Date:     Date.FormatByTimespan(it.Date),
			FileFlag: it.StorageId != 0,
			Thumb:    Bool.Is(it.HasThumb, "/app/files/thumb/"+String.ValueOf(it.Id), ""),
		}
		forms = append(forms, outForm)
	}
	return forms
}

//
//    @Operation(summary = "获取扩展文件的所有key值")
//    @PostMapping("/get_extra_keys")
//    @ResponseBody
//    fun getExtraKeys(
//        @Parameter(name = "文件id") @RequestParam("id", required = true) id: Long
//    ): List<String> {
//        return this.dfsFileDao.selectExtraNames(id)
//    }

// 创建文件夹
// @Post:/create_folder
func CreateFolder(folder string) error {
	loginId := LoginState.LoginId()
	nameList, err := String.ToDfsFileNameList(folder)
	if err != nil {
		return err
	}
	existsFileId := DfsFileDao.SelectIdByPath(loginId, nameList)
	if existsFileId != 0 {
		return exception.EXISTS(folder)
	}
	DfsFileService.Mkdirs(loginId, folder)
	return nil
}

// 删除文件
// @Post:/delete
func Delete(paths []string) error {
	loginId := LoginState.LoginId()
	for _, it := range paths {
		err := DfsFileService.SetDelete(loginId, it)
		if err != nil {
			return err
		}
	}
	return nil
}

// 重命名
// sourcePath 源路径
// name 新名称
// @Post:/rename
func Rename(sourcePath string, name string) error {
	if strings.Contains(name, "/") {
		return exception.Biz("文件名不能包含/")
	}
	if strings.Contains(name, "\\") {
		return exception.Biz("文件名不能包含\\")
	}
	loginId := LoginState.LoginId()
	return DfsFileService.Rename(loginId, sourcePath, name)
}

// 文件复制
// sourcePaths 源路径
// targetFolder 目标文件夹
// isOverWrite 是否覆盖目标文件
// @Post:/copy
func Copy(sourcePaths []string, targetFolder string, isOverWrite bool) error {
	loginId := LoginState.LoginId()
	return DfsFileService.Copy(loginId, sourcePaths, targetFolder, isOverWrite)
}

// 文件移动
// sourcePaths 源路径
// targetFolder 目标文件夹
// isOverWrite 是否覆盖目标文件
// @Post:/move
func Move(sourcePaths []string, targetFolder string, isOverWrite bool) error {
	loginId := LoginState.LoginId()
	return DfsFileService.Move(loginId, sourcePaths, targetFolder, isOverWrite)
}

// 分享文件
// @Post:/share
func Share(inForm form.ShareForm) any {
	loginId := LoginState.LoginId()
	shareId, err := FileShareService.Share(loginId, inForm)
	if err != nil {
		return err
	}
	return shareId
}

// 文件或文件夹属性
// paths 选择的路径列表
// @Post:/get_property
func GetProperty(paths []string) any {
	loginId := LoginState.LoginId()
	outForm := form.FilePropertyForm{}
	if len(paths) > 1 { //多个文件时
		totalForm := form.ComputeSubTotalForm{}
		for _, it := range paths {
			nameList, _ := String.ToDfsFileNameList(it)
			fileId := DfsFileDao.SelectIdByPath(loginId, nameList)
			if fileId == 0 {
				return exception.NO_EXISTS()
			}
			dfsFile, _ := DfsFileDao.SelectOne(fileId)
			if dfsFile.IsFolder() {
				totalForm.FolderCount += 1
				computeSubTotal(&totalForm, loginId, dfsFile.Id)
			} else {
				totalForm.FileCount += 1
				totalForm.Size += dfsFile.Size
			}
		}
		outForm.Size = Number.ToDataSize(totalForm.Size)
		outForm.FileCount = totalForm.FileCount
		outForm.FolderCount = totalForm.FolderCount
	} else { //单文件时
		path := paths[0]
		nameList, _ := String.ToDfsFileNameList(path)
		fileId := DfsFileDao.SelectIdByPath(loginId, nameList)
		if fileId == 0 {
			return exception.NO_EXISTS()
		}
		dfsFile, _ := DfsFileDao.SelectOne(fileId)
		outForm.Name = dfsFile.Name
		outForm.Date = Date.FormatByTimespan(dfsFile.Date)
		outForm.Path = path
		outForm.IsFile = dfsFile.IsFile()
		if dfsFile.IsFile() { //文件时
			outForm.Size = Number.ToDataSize(dfsFile.Size)
			outForm.ContentType = dfsFile.ContentType
			historyList := make([]form.FilePropertyHistoryForm, 0)
			for _, it := range DfsFileDao.SelectHistory(loginId, fileId) {
				hForm := form.FilePropertyHistoryForm{
					Id:   it.Id,
					Size: Number.ToDataSize(it.Size),
					Date: Date.FormatByTimespan(it.Date),
				}
				historyList = append(historyList, hForm)
			}
			outForm.HistoryList = historyList
		} else { //文件夹时
			totalForm := form.ComputeSubTotalForm{}
			computeSubTotal(&totalForm, loginId, dfsFile.Id)
			outForm.FileCount = totalForm.FileCount
			outForm.FolderCount = totalForm.FolderCount
			outForm.Size = Number.ToDataSize(totalForm.Size)
		}
	}
	return outForm
}

// 计算文件大小
func computeSubTotal(totalForm *form.ComputeSubTotalForm, loginId int64, folderId int64) {
	subList := DfsFileDao.SelectSubFile(loginId, folderId)
	for _, it := range subList {
		if it.StorageId == 0 {
			totalForm.FolderCount += 1
			computeSubTotal(totalForm, loginId, it.Id)
		} else {
			totalForm.FileCount += 1
			totalForm.Size += it.Size
		}
	}
}

// 修改文件类型
// path 文件路径
// contentType 文件类型
// @Post:/set_content_type
func SetContentType(path string, contentType string) error {
	loginId := LoginState.LoginId()
	nameList, _ := String.ToDfsFileNameList(path)
	fileId := DfsFileDao.SelectIdByPath(loginId, nameList)
	if fileId == 0 {
		return exception.NO_EXISTS()
	}
	DfsFileDao.SetContentType(fileId, contentType)
	return nil
}

/**
 * 文件下载
 * @param request 客户端请求
 * @param response 往客户端返回内容
 * @param id 文件ID
 */
//@Get:/download_history/
func DownloadByHistory(writer http.ResponseWriter, request *http.Request, id int64) {
	loginId := LoginState.LoginId()
	fileName := request.URL.Path
	fileName = fileName[strings.Index(fileName, "/download_history")+18:]
	dfsFile, isExists := DfsFileDao.SelectOne(id)
	if !isExists { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	if dfsFile.UserId != loginId { // 没有操作权限
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	if dfsFile.Name != fileName {
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	DfsFileUtil.DownloadDfs(dfsFile, writer, request)
}

// 文件预览
// dfsId dfs文件ID
// name 文件名
// extra 要预览的附属文件名
// @Get:/preview/{dfsId}/{name}
func Preview(writer http.ResponseWriter, request *http.Request, dfsId int64, name string, extra string) {
	loginId := LoginState.LoginId()
	dfsDto, isExists := DfsFileDao.SelectOne(dfsId)
	if !isExists { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	if dfsDto.UserId != loginId { // 没有操作权限
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	if dfsDto.Name != name { // 文件名不一致，没有操作权限
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	//在头部信息中加入文件名
	//writer.Header().Set("File-Name", name)

	if extra == "" { //下载源文件
		DfsFileUtil.DownloadDfs(dfsDto, writer, request)
		return
	}
	previewDto, isExists := DfsFileDao.SelectExtra(dfsId, extra)
	if isExists { //存在预览文件
		DfsFileUtil.DownloadDfs(previewDto, writer, request)
		return
	}
	writer.WriteHeader(http.StatusNotFound)
}

// 文件下载
// name 文件名
// folder 所在文件夹
// @TODO:这里应该改成文件id访问，防止客户端缓存冲突
// @Get:/download/
func Download(writer http.ResponseWriter, request *http.Request) {
	loginId := LoginState.LoginId()
	filePath := request.URL.Path
	filePath = filePath[strings.Index(filePath, "/download")+9:]
	nameList, _ := String.ToDfsFileNameList(filePath)
	fileId := DfsFileDao.SelectIdByPath(loginId, nameList)
	if fileId == 0 { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	DfsFileUtil.DownloadDfsId(fileId, writer, request)
}

// 缩略图下载
// id 文件ID
// @Request:/thumb/{id}
func Thumb(writer http.ResponseWriter, request *http.Request, id int64) {
	dfsDto, isExists := DfsFileDao.SelectOne(id)
	if !isExists { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	loginId := LoginState.LoginId()
	if dfsDto.UserId != loginId { //没有权限
		writer.WriteHeader(http.StatusForbidden)
		return
	}

	//获取缩率图附属文件
	thumb, isExists := DfsFileDao.SelectExtra(dfsDto.Id, "thumb")
	DfsFileUtil.DownloadDfs(thumb, writer, request)
}
