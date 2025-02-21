package UserService

import (
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"time"
)

/**
 * 用户操作Service
 */

/**
 * 添加一个用户
 * @param dto 用户Dto
 */
func Add(dto dto.UserDto) {
	dto.Date = time.Now().UnixMilli()
	UserDao.Add(dto)
}
