package UserService

import (
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Number"
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
	date := time.Now()
	id := Number.ID()
	dto.Date = date
	dto.Id = id
	UserDao.Add(dto)
}
