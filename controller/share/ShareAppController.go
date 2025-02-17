package share

import (
	"DairoDFS/controller/share/form"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/ShareDao"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DfsFileHandleUtil"
	"DairoDFS/util/DfsFileUtil"
	"errors"
	"net/http"
	"strings"
	"time"
)

// 提取分享的文件
//@Group:/share/{id}

// 页面初始化
// @Get:/init.html
// @Html:share/share.html
func Html(writer http.ResponseWriter, request *http.Request, id int64) {
	_, err := getShare(writer, request, id)
	if err == nil {
		return
	}
	var bizErr *exception.BusinessException
	errors.As(err, &bizErr)
	if bizErr.Code == exception.SHARE_NEED_PWD().Code { // 跳转输入密码页面
		http.Redirect(writer, request, "/share/"+String.ValueOf(id)+"/input_pwd", http.StatusFound)
	} else {

	}
	//} catch (e: BusinessException) {
	//    return when (e.code) {
	//        ErrorCode.SHARE_NEED_PWD.code -> "app/share_pwd"
	//        else -> {
	//            model.addAttribute("error", e.message)
	//            "app/share_error"
	//        }
	//    }
	//}
}

// 输入密码
// @Get:/input_pwd
// @Html:share/share_pwd.html
func InputPwd(id int64) {

}

// 验证密码
// id 分享ID
// @Post:/valid_pwd
func ValidPwd(writer http.ResponseWriter, request *http.Request, id int64, pwd string) any {
	shareDto, isExists := ShareDao.SelectOne(id)
	if !isExists { //分享不存在
		return exception.SHARE_NOT_FOUND()
	}
	if pwd == shareDto.Pwd { //密码验证成功,返回加密密码
		return makeEncodePwd(writer, request, pwd)
	} else {
		return exception.Biz("密码不正确")
	}
}

/**
 * 转存
 * @param id 分享ID
 * @param folder 所选择的父级文件夹
 * @param names 所选择的文件夹或文件名数组
 * @param target 要转存的目标文件夹
 */
//@Post:/save_to
func SaveTo(writer http.ResponseWriter, request *http.Request, id int64, folder string, names []string, target string) error {

	//获取APP登录票据
	cookieToken, _ := request.Cookie("token")
	if cookieToken == nil {
		return nil
	}
	token := cookieToken.Value
	if len(token) == 0 {
		return nil
	}
	userId := UserTokenDao.GetByUserIdByToken(token)
	if userId == 0 { //用户未登录
		return nil
	}

	sharePaths := make([]string, 0)
	for _, it := range names {
		sharePaths = append(sharePaths, folder+"/"+it)
	}
	shareDto, validateErr := validateShare(writer, request, id, sharePaths...)
	if validateErr != nil {
		return validateErr
	}

	// 文件真实目录列表
	truePaths := make([]string, 0)
	for _, it := range names {
		truePaths = append(truePaths, shareDto.Folder+folder+"/"+it)
	}
	if err := DfsFileService.ShareSaveTo(shareDto.UserId, userId, truePaths, target, false); err != nil {
		return err
	}

	//开启生成缩略图线程
	DfsFileHandleUtil.NotifyWorker()
	return nil
}

// GetList 重置密码
// id 分享ID
// folder 分享的文件夹路径
// @Post:/get_list
func GetList(writer http.ResponseWriter, request *http.Request, id int64, folder string) any {
	shareDto, validateErr := validateShare(writer, request, id, folder)
	if validateErr != nil {
		return validateErr
	}

	//用户ID
	userId := shareDto.UserId

	//文件列表
	fileList := make([]dto.DfsFileThumbDto, 0)
	if folder == "" { //所分享目录的根目录，并非用户跟目录

		//得到分享的父文件夹ID
		shareFolderId, folderIdErr := DfsFileService.GetIdByFolder(userId, shareDto.Folder, false)
		if folderIdErr != nil {
			return folderIdErr
		}

		//分享的文件名或文件夹名列表
		shareFileNameMap := make(map[string]struct{})
		for _, it := range strings.Split(shareDto.Names, "|") {
			shareFileNameMap[it] = struct{}{}
		}

		//需要筛选出分享的文件
		for _, it := range DfsFileDao.SelectSubFile(userId, shareFolderId) {
			_, isExists := shareFileNameMap[it.Name]
			if isExists {
				fileList = append(fileList, it)
			}
		}
	} else {

		//实际文件夹目录
		trueFolder := shareDto.Folder + folder

		//得到分享的父文件夹ID
		folderId, folderIdErr := DfsFileService.GetIdByFolder(userId, trueFolder, false)
		if folderIdErr != nil {
			return folderIdErr
		}
		//?: throw ErrorCode.NO_FOLDER
		fileList = DfsFileDao.SelectSubFile(userId, folderId)
	}
	formList := make([]form.ShareForm, 0)
	for _, it := range fileList {
		outForm := form.ShareForm{
			Name:     it.Name,
			Size:     it.Size,
			Date:     Date.FormatByTimespan(it.Date),
			FileFlag: it.StorageId > 0,
			Thumb:    Bool.Is(it.HasThumb, "thumb?fid="+String.ValueOf(it.Id), ""),
		}
		formList = append(formList, outForm)
	}
	return formList
}

// Download 文件下载
// request 客户端请求
// response 往客户端返回内容
// id 分享ID
// name 文件名
// folder 所在文件夹
// @Get:/download/{name}
func Download(writer http.ResponseWriter, request *http.Request, id int64, folder string, name string) error {
	path := folder + "/" + name
	shareDto, validateErr := validateShare(writer, request, id, path)
	if validateErr != nil {
		return validateErr
	}

	//用户ID
	userId := shareDto.UserId

	//实际文件目录
	truePath := shareDto.Folder + path

	names, _ := String.ToDfsFileNameList(truePath)

	//得到文件ID
	fileId := DfsFileDao.SelectIdByPath(userId, names)
	DfsFileUtil.DownloadDfsId(fileId, writer, request)
	return nil
}

/**
 * 分享的文件路径验证，避免暴露未分享的文件
 * @param id 分享ID
 * @param path 分享的路径数组
 */
func validateShare(writer http.ResponseWriter, request *http.Request, id int64, paths ...string) (dto.ShareDto, error) {
	shareDto, getErr := getShare(writer, request, id)
	if getErr != nil {
		return dto.ShareDto{}, getErr
	}

	//得到分享的父文件夹ID
	shareFolderId, folderErr := DfsFileService.GetIdByFolder(shareDto.UserId, shareDto.Folder, false)
	if folderErr != nil {
		return dto.ShareDto{}, exception.NO_FOLDER()
	}

	if shareFolderId > 0 { //非根目录时,要验证是否存在文件夹
		dfsFile, isExists := DfsFileDao.SelectOne(shareFolderId)
		if !isExists {
			return dto.ShareDto{}, exception.NO_FOLDER()
		}
		if dfsFile.DeleteDate != 0 { //文件已经删除
			return dto.ShareDto{}, exception.NO_FOLDER()
		}
	}

	//分享的文件名或文件夹名列表
	shareNameSet := make(map[string]struct{})
	for _, it := range strings.Split(shareDto.Names, "|") {
		shareNameSet[it] = struct{}{}
	}
	for _, it := range paths {
		if it == "" {
			continue
		}
		names, toNamesErr := String.ToDfsFileNameList(it)
		if toNamesErr != nil {
			return dto.ShareDto{}, toNamesErr
		}
		if _, isExists := shareNameSet[names[1]]; !isExists {
			return dto.ShareDto{}, exception.NO_FOLDER()
		}
	}
	return shareDto, nil
}

// 缩略图
// request 客户端请求
// response 往客户端返回内容
// id 文件ID
// @Get:/thumb
func Thumb(writer http.ResponseWriter, request *http.Request, fid int64) {
	//shareDto, _ := getShare(writer, request, id)
	//val dfsId = AESUtil.decrypt(tag.replace(" ", "+"), id.toString())
	//dfsDto = DfsFileDao.SelectOne(dfsId!!.toLong())
	dfsDto, isExists := DfsFileDao.SelectOne(fid)
	if !isExists { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}

	//获取缩率图附属文件
	thumb, _ := DfsFileDao.SelectExtra(dfsDto.Id, "thumb")
	DfsFileUtil.DownloadDfs(thumb, writer, request)
}

// 获取分享的信息
// id 分享ID
// return 分享信息
func getShare(writer http.ResponseWriter, request *http.Request, id int64) (dto.ShareDto, error) {
	shareDto, isExists := ShareDao.SelectOne(id)
	if !isExists { //分享链接不存在
		return dto.ShareDto{}, exception.SHARE_NOT_FOUND()
	}
	if shareDto.EndDate != 0 { //如果有结束日期
		if shareDto.EndDate < time.Now().UnixMilli() { //分享已过期
			return dto.ShareDto{}, exception.SHARE_IS_END()
		}
	}
	if shareDto.Pwd == "" { //如果不需要提取码
		return shareDto, nil
	}

	//从cookie中获取加密提取码
	cookieEncodePwd, _ := request.Cookie("share_code")
	if cookieEncodePwd == nil {
		return dto.ShareDto{}, exception.SHARE_NEED_PWD()
	}
	encodePwd := cookieEncodePwd.Value
	if encodePwd != makeEncodePwd(writer, request, shareDto.Pwd) {
		return dto.ShareDto{}, exception.SHARE_NEED_PWD()
	}
	return shareDto, nil
}

// 生成加密提取码
func makeEncodePwd(writer http.ResponseWriter, request *http.Request, pwd string) string {
	return pwd
}
