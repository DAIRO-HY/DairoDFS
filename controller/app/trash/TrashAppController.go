package trash

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/trash/form"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileDeleteService"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/LoginState"
	"time"
)

/**
 * 垃圾桶文件列表
 */
//@Group:/app/trash

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

// 获取回收站文件列表
// @Post:/get_list
func GetList() []form.TrashForm {
	loginId := LoginState.LoginId()
	now := time.Now().UnixMilli()
	var trashSaveTime int64 = application.TRASH_TIMEOUT * 24 * 60 * 60 * 1000
	list := make([]form.TrashForm, 0)
	for _, it := range DfsFileDao.SelectDelete(loginId) {
		deleteDate := it.DeleteDate

		//剩余删除时间
		timeLeft := trashSaveTime - (now - deleteDate)

		var deleteLastTime string
		if timeLeft < 0 {
			deleteLastTime = "即将删除"
		} else {
			if timeLeft > 24*60*60*1000 { //超过1天
				deleteLastTime = String.ValueOf(timeLeft/(24*60*60*1000)) + "天后删除"
			} else if timeLeft > 60*60*1000 { //超过1小时
				deleteLastTime = String.ValueOf(timeLeft/(60*60*1000)) + "小时后删除"
			} else if timeLeft > 60*1000 { //超过1分钟
				deleteLastTime = String.ValueOf(timeLeft/(60*1000)) + "分钟后删除"
			} else {
				deleteLastTime = "即将删除"
			}
		}
		outForm := form.TrashForm{
			Id:       it.Id,
			Name:     it.Name,
			Size:     it.Size,
			Date:     deleteLastTime,
			FileFlag: it.StorageId > 0,
			Thumb:    Bool.Is(it.HasThumb, "/app/files/thumb/"+String.ValueOf(it.Id), ""),
		}
		list = append(list, outForm)
	}
	return list
}

// 彻底删除文件
// ids 选中的文件ID列表
// @Post:/logic_delete
func LogicDelete(ids []int64) error {
	loginId := LoginState.LoginId()
	for _, it := range ids { //验证是否有删除权限
		fileDto, _ := DfsFileDao.SelectOne(it)
		if fileDto.UserId != loginId { //非自己的文件，无法删除
			return exception.NOT_ALLOW()
		}
		if fileDto.DeleteDate == 0 { //该文件未标记为删除
			return exception.NOT_ALLOW()
		}
	}
	DfsFileDeleteService.AddDelete(ids)
	return nil
}

// 从垃圾箱还原文件
// ids 选中的文件ID列表
// @Post:/trash_recover
func TrashRecover(ids []int64) error {
	loginId := LoginState.LoginId()
	return DfsFileService.TrashRecover(loginId, ids)
}

// 立即回收储存空间
// @Post:/recycle_storage
func RecycleStorage() {

	//@TODO:待实现
	//只有管理员才有权限操作
	//if (request.getAttribute(Constant.REQUEST_IS_ADMIN) as Boolean) {//强制删除
	//    Constant.dbService.exec("update dfs_file_delete set deleteDate = 0 where 1=1")
	//    RecycleStorageTimer::class.bean.recycle()
	//}
}
