package my_share

import (
	"DairoDFS/controller/app/my_share/form"
	"DairoDFS/dao/ShareDao"
	"DairoDFS/exception"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/String"
	"DairoDFS/util/AESUtil"
	"DairoDFS/util/LoginState"
	"time"
)

/**
 * 提取分享的文件
 */
//@Group:/app/my_share

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

// 获取所有的分享
// @Post:/get_list
func GetList() []form.MyShareForm {
	loginId := LoginState.LoginId()
	now := time.Now().UnixMilli()
	list := make([]form.MyShareForm, 0)
	for _, it := range ShareDao.SelectByUser(loginId) {
		endDate := ""
		if it.EndDate == 0 {
			endDate = "永久有效"
		} else {
			endTime := it.EndDate - now
			if endTime < 0 {
				endDate = "已过期"
			} else {
				if endTime > 24*60*60*1000 { //超过1天
					endDate = String.ValueOf(endTime/(24*60*60*1000)) + "天后过期"
				} else if endTime > 60*60*1000 { //超过1小时
					endDate = String.ValueOf(endTime/(60*60*1000)) + "小时后过期"
				} else if endTime > 60*1000 { //超过1分钟
					endDate = String.ValueOf(endTime/(60*1000)) + "分钟后过期"
				} else {
					endDate = "即将过期"
				}
			}
		}
		shareForm := form.MyShareForm{
			Id:         it.Id,
			Title:      it.Title,
			FileCount:  it.FileCount,
			FolderFlag: it.FolderFlag,
			EndDate:    endDate,
			Thumb:      Bool.Is(it.Thumb != 0, "/app/files/thumb/"+String.ValueOf(it.Thumb), ""),
			Date:       Date.FormatByTimespan(it.Date),
		}
		list = append(list, shareForm)
	}
	return list
}

// 获取分享详细
// id 分享id
// @Post:/get_detail
func GetDetail(id int64) form.MyShareDetailForm {
	loginId := LoginState.LoginId()
	shareDto, _ := ShareDao.SelectOne(id)
	if shareDto.UserId != loginId {
		panic(exception.NOT_ALLOW())
	}
	folder := shareDto.Folder
	if folder == "" {
		folder = "/"
	}
	endDate := ""
	if shareDto.EndDate == 0 {
		endDate = "永久有效"
	} else {
		endDate = Date.FormatByTimespan(shareDto.EndDate)
	}

	//分享链接
	url := "/share/" + AESUtil.Encrypt(String.ValueOf(shareDto.Id)) + "/init"

	outForm := form.MyShareDetailForm{
		Id:      shareDto.Id,
		Url:     url,
		Names:   shareDto.Names,
		Date:    Date.FormatByTimespan(shareDto.Date),
		EndDate: endDate,
		Pwd:     Bool.Is(shareDto.Pwd == "", "无", shareDto.Pwd),
		Folder:  folder,
	}
	return outForm
}

// 取消所选分享
// @Post:/delete
// ids 分享id列表
func Delete(ids []int64) {
	loginId := LoginState.LoginId()
	idsStr := ""
	for _, it := range ids {
		idsStr += String.ValueOf(it) + ","
	}
	idsStr = idsStr[:len(idsStr)-1]
	ShareDao.Delete(loginId, idsStr)
}
