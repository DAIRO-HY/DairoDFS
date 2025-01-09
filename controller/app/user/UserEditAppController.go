package user

/**
 * 用户编辑
 */
import (
	"DairoDFS/controller/app/user/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Date"
	"DairoDFS/service/UserService"
)

const PWD_PLACEHOLDER = "********************************"

/**
 * 初始化
 */
//@Get:/app/user_edit
//@templates:app/user_edit.html
func EditHtml() {}

/**
 * 页面初始化
 */
//@Post:/app/user_edit/init
func EditInit(id int64) form.UserEditInoutForm {
	if id != 0 {
		userDto, _ := UserDao.SelectOne(id)
		date := Date.Format(userDto.Date)
		pwd := PWD_PLACEHOLDER
		return form.UserEditInoutForm{
			Id:    userDto.Id,
			Name:  userDto.Name,
			Pwd:   pwd,
			Email: userDto.Email,
			Date:  date,
			State: userDto.State,
		}
	} else {
		return form.UserEditInoutForm{}
	}
}

/**
 * 添加或更新数据
 */
//@Post:/app/user_edit/edit
func Edit(inForm form.UserEditInoutForm) {
	userDto := dto.UserDto{
		Id:    inForm.Id,
		Name:  inForm.Name,
		Email: inForm.Email,
		State: inForm.State,
	}
	if inForm.Id == 0 {
		UserService.Add(userDto)
	} else {
		UserDao.Update(userDto)
	}
	if inForm.Pwd != PWD_PLACEHOLDER { //更新密码
		UserDao.SetPwd(inForm.Id, inForm.Pwd)
	}
}
