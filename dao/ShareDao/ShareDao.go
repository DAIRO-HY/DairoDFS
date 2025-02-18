package ShareDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 */
func Add(dto dto.ShareDto) {
	DBUtil.InsertIgnoreError(`insert into share(id, title, userId, pwd, folder, names, thumb, folderFlag, fileCount, endDate, date)
        values (?,?,?,?,?,?,?,?,?,?,?)`, dto.Id, dto.Title, dto.UserId, dto.Pwd, dto.Folder, dto.Names, dto.Thumb, dto.FolderFlag, dto.FileCount, dto.EndDate, dto.Date)
}

/**
 * 通过ID获取一条数据
 */
func SelectOne(id int64) (dto.ShareDto, bool) {
	return DBUtil.SelectOne[dto.ShareDto]("select * from share where id = ?", id)
}

/**
 * 获取所有分享列表
 */
func SelectByUser(userId int64) []dto.ShareDto {
	return DBUtil.SelectList[dto.ShareDto](`select id, title, pwd, folder, thumb, folderFlag, fileCount, endDate, date
        from share where userId = ?`, userId)
}

/**
 * 删除分享
 * @param userId 用户ID
 * @param ids 要删除的分享id列表
 */
func Delete(userId int64, ids string) {
	DBUtil.ExecIgnoreError("delete from share where userId = ? and id in ("+ids+")", userId)
}
