package DfsFileDeleteService

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/DfsFileDeleteDao"
	"DairoDFS/dao/dto"
	"time"
)

// 文件删除操作Service

/**
 * 彻底删除文件
 * @param ids 要删除的文件ID
 */
func AddDelete(ids []int64) {
	for _, it := range ids {
		fileDto := DfsFileDao.SelectOne(it)
		if fileDto.IsFolder() { //如果是文件夹
			deleteFolder(*fileDto)
		} else {
			//彻底删除文件
			deleteSelfAndExtra(*fileDto)
		}
	}
}

/**
 * 递归删除文件夹所有类容
 * @param fileDto 要删除的文件
 */
func deleteFolder(fileDto dto.DfsFileDto) {
	for _, it := range DfsFileDao.SelectAllChildList(*fileDto.Id) {
		if it.IsFolder() {
			deleteFolder(*it)
		} else {
			deleteSelfAndExtra(*it)
		}
	}

	//彻底删除文件夹
	DfsFileDao.Delete(*fileDto.Id)
}

/**
 * 删除文件本身和附属文件
 */
func deleteSelfAndExtra(fileDto dto.DfsFileDto) {
	if !*fileDto.IsExtra { //如果这不是一个附属文件

		//获取附属文件
		extraList := DfsFileDao.SelectExtraListById(*fileDto.Id)
		for _, it := range extraList { //删除文件所有附属文件
			addDelete(*it)
		}
	}
	addDelete(fileDto)
}

/**
 * 彻底删除文件
 */
func addDelete(fileDto dto.DfsFileDto) {
	DfsFileDeleteDao.Insert(*fileDto.Id)
	DfsFileDeleteDao.SetDeleteDate(*fileDto.Id, time.Now().UnixMilli())
	DfsFileDao.Delete(*fileDto.Id)
}
