package UserService

import (
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Number"
	"DairoDFS/service/DfsFileService"
	"time"
)

/**
 * 添加一个用户
 * @param dto 用户Dto
 */
func Add(user dto.UserDto) {
	user.Date = time.Now().UnixMilli()
	UserDao.Add(user)

	//创建用户时，为用户添加一个根目录
	createFolderDto := dto.DfsFileDto{
		Id:       Number.ID(),
		UserId:   user.Id,
		Name:     "",
		ParentId: 0,
		Size:     0,
	}
	DfsFileService.AddFolder(createFolderDto)
}
