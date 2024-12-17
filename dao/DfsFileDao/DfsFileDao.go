package DfsFileDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
	"strconv"
)

/**
 * 添加一条数据
 */
func Add(fileDto dto.DfsFileDto) {
	DBUtil.InsertIgnoreError("insert into dfs_file(id, userId, parentId, name, size, contentType, localId, date, isExtra, property, state) values (?,?,?,?,?,?,?,?,?,?,?)",
		fileDto.Id,
		fileDto.UserId,
		fileDto.ParentId,
		fileDto.Name,
		fileDto.Size,
		fileDto.ContentType,
		fileDto.LocalId,
		fileDto.Date,
		fileDto.IsExtra,
		fileDto.Property,
		fileDto.State)
}

/**
 * 通过id获取一条数据
 * @param id 文件ID
 */
func SelectOne(id int64) *dto.DfsFileDto {
	return DBUtil.SelectOne[dto.DfsFileDto]("select * from dfs_file where id = ?", id)
}

/**
 * 通过文件夹ID和文件名获取文件信息
 * @param parentId 文件夹ID
 * @param name 文件名
 * @return 文件信息
 */
func SelectByParentIdAndName(userId int64, parentId int64, name string) *dto.DfsFileDto {
	return DBUtil.SelectOne[dto.DfsFileDto](
		`select * from dfs_file where userId = ?
          and parentId = ?
          and name COLLATE NOCASE = ?
          and isHistory = 0
          and deleteDate is null`, userId, parentId, name)
}

/**
 * 通过文件夹ID和文件名获取文件Id
 * @param parentId 文件夹ID
 * @param name 文件名
 * @return 文件信息
 */
func SelectIdByParentIdAndName(userId int64, parentId int64, name string) int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64](`select id from dfs_file where userId = ?
          and parentId = ?
          and name COLLATE NOCASE = ?
          and isHistory = 0
          and deleteDate is null`, userId, parentId, name)
}

/**
 * 通过路径获取文件ID
 * @param names 文件名列表
 * @return 文件ID
 */
func SelectIdByPath(userId int64, names []string) *int64 {
	sql := ""
	for range names {
		sql += "select id from dfs_file where userId = " + strconv.FormatInt(userId, 10) + " and parentId = ("
	}
	sql += "0"
	for _, name := range names {
		sql += ") and name COLLATE NOCASE = '" + name + "' and isHistory = 0 and deleteDate is null"
	}
	return DBUtil.SelectSingleOneIgnoreError[int64](sql)
}

/**
 * 获取子文件id和文件名
 * @param parentId 文件夹id
 * @return 子文件列表
 */
func SelectSubFileIdAndName(userId int64, parentId int64) []*dto.DfsFileDto {
	return DBUtil.SelectList[dto.DfsFileDto](`select id, name, localId from dfs_file where userId = ?
          and parentId = ?
          and isHistory = 0
          and deleteDate is null`, userId, parentId)
}

/**
 * 获取子文件信息,客户端显示用
 * @param parentId 文件夹id
 * @return 子文件列表
 */
func SelectSubFile(userId int64, parentId int64) []*dto.DfsFileThumbDto {
	return DBUtil.SelectList[dto.DfsFileThumbDto](`select df.id, df.name, df.size, df.date, df.localId, thumbDf.id > 0 as hasThumb
        from dfs_file as df
                 left join dfs_file as thumbDf
                           on thumbDf.parentId = df.id and df.localId > 0 and thumbDf.name = 'thumb'
        where df.userId = ?
          and df.parentId = ?
          and df.isHistory = 0
          and df.deleteDate is null`, userId, parentId)
}

/**
 * 获取全部已经删除的文件
 * @param userId 用户ID
 * @return 已删除的文件
 */
func SelectDelete(userId int64) []*dto.DfsFileThumbDto {
	sql := `select df.id, df.name, df.size, df.localId, df.deleteDate, thumbDf.id > 0 as hasThumb
        from dfs_file as df
                 left join dfs_file as thumbDf
                           on thumbDf.parentId = df.id and df.localId > 0 and thumbDf.name = 'thumb'
        where df.userId = ?
          and df.isHistory = 0
          and df.deleteDate is not null`
	return DBUtil.SelectList[dto.DfsFileThumbDto](sql, userId)
}

/**
 * 获取所有回收站超时的数据
 * @return 已删除的文件
 */
func SelectIdsByDeleteAndTimeout(time int64) []*int64 {
	sql := "select id from dfs_file where deleteDate < ? limit 1000"
	return DBUtil.SelectList[int64](sql, time)
}

/**
 * 获取文件历史版本
 * @param userId 用户ID
 * @param id 文件id
 * @return 历史版本列表
 */
func SelectHistory(userId int64, id int64) []*dto.DfsFileDto {
	sql := `select id, size, date from dfs_file where userId = ?
          and parentId = (select parentId from dfs_file where id = ?)
          and name = (select name from dfs_file where id = ?)
          and isHistory = 1
          and deleteDate is null`
	return DBUtil.SelectList[dto.DfsFileDto](sql, userId, id, id)
}

/**
 * 获取尚未处理的数据
 */
func SelectNoHandle() []*dto.DfsFileDto {
	return DBUtil.SelectList[dto.DfsFileDto]("select * from dfs_file where localId > 0 and state = 0 order by id asc limit 1000")
}

/**
 * 将文件标记为历史版本
 * @param id 文件ID
 */
func SetHistory(id int64) {
	DBUtil.ExecIgnoreError("update dfs_file set isHistory = 1 where id = ?", id)
}

/**
 * 将文件标记为删除
 * @param id 文件ID
 * @param time 时间戳
 */
func SetDelete(id int64, time int64) {
	sql := `update dfs_file set deleteDate = ? where userId = (select userId from dfs_file where id = ?)
          and parentId = (select parentId from dfs_file where id = ?)
          and name = (select name from dfs_file where id = ?)
          and deleteDate is null`
	DBUtil.ExecIgnoreError(sql, time, id, id, id)
}

/**
 * 将标记为删除文件还原
 * @param id 文件ID
 */
func SetNotDelete(id int64) {
	sql := `update dfs_file set deleteDate = null where userId = (select userId from dfs_file where id = ?)
          and parentId = (select parentId from dfs_file where id = ?)
          and name = (select name from dfs_file where id = ?)
          and deleteDate = (select deleteDate from dfs_file where id = ?)`
	DBUtil.ExecIgnoreError(sql, id, id, id, id)
}

/**
 * 修改文件类型
 * @param id 文件ID
 */
func SetContentType(id int64, contentType string) {
	DBUtil.ExecIgnoreError("update dfs_file set contentType = ? where id = ? and localId > 0", contentType, id)
}

/**
 * 删除
 * @param id 文件ID
 */
func Delete(id int64) {
	DBUtil.ExecIgnoreError("delete from dfs_file where id = ?", id)
}

/**
 * 文件移动
 * @param dto 移动文件信息
 */
func Move(dto dto.DfsFileDto) {
	DBUtil.ExecIgnoreError("update dfs_file set parentId = ?, name = ? where id = ?", dto.ParentId, dto.Name, dto.Id)
}

/**
 * 设置文件属性
 */
func SetProperty(id int64, property string) {
	DBUtil.ExecIgnoreError("update dfs_file set property = ? where id = ?", property, id)
}

/**
 * 设置文件处理状态
 */
func SetState(id int64, state int8, stateMsg string) {
	DBUtil.ExecIgnoreError("update dfs_file set state = ?, stateMsg = ? where id = ?", state, stateMsg, id)
}

/**
 * 验证文件存储ID权限
 */
func ValidLocalId(userId int64, localId int64) bool {
	return DBUtil.SelectSingleOneIgnoreError[bool]("select count(*) > 0 from dfs_file where userId = ? and localId = ?", userId, localId)
}

/**
 * 获取附属文件
 * @param parentId dfs文件ID
 * @param name 附属文件标题
 * @return 附属文件信息
 */
func SelectExtra(parentId int64, name string) *dto.DfsFileDto {
	sql := `select * from dfs_file where parentId = ? and name = ? and isExtra = 1`
	return DBUtil.SelectOne[dto.DfsFileDto](sql, parentId, name)
}

/**
 * 获取扩展文件的所有key值
 * @param id dfs文件ID
 * @return 附属文件信息
 */
func SelectExtraNames(id int64) []string {
	sql := "select name from dfs_file where parentId = ? and isExtra = 1"
	return DBUtil.SelectList[string](sql, id)
}

/**
 * 通过本地存储ID查询文件属性
 * @param localId 本地存储id
 * @return 属性
 */
func SelectPropertyByLocalId(localId int64) string {
	sql := "select property from dfs_file where localId = ? and state = 1 and property is not null limit 1"
	return DBUtil.SelectSingleOneIgnoreError[string](sql, localId)
}

/**
 * 通过本地存储ID查询文件附属文件
 * @param localId 本地存储id
 * @return 附属文件列表
 */
func SelectExtraFileByLocalId(localId int64) []*dto.DfsFileDto {
	sql := `select * from dfs_file where parentId = (select id from dfs_file where localId = ? and state = 1 limit 1) and isExtra = 1`
	return DBUtil.SelectList[dto.DfsFileDto](sql, localId)
}

/**
 * 获取某个文件附属文件
 * @param id 文件id
 * @return 附属文件列表
 */
func SelectExtraListById(id int64) []*dto.DfsFileDto {
	sql := "select * from dfs_file where parentId = ? and isExtra = 1"
	return DBUtil.SelectList[dto.DfsFileDto](sql, id)
}

/**
 * 获取某个文件夹下的所有文件及文件夹，包括历史文件，已删除文件
 * @param id 文件id
 * @return 文件夹下的所有文件及文件夹，包括历史文件，已删除文件
 */
func SelectAllChildList(id int64) []*dto.DfsFileDto {
	sql := `select * from dfs_file where parentId = ?`
	return DBUtil.SelectList[dto.DfsFileDto](sql, id)
}

/**
 * 文件是否正在使用中
 * @param id 本地文件id
 */
func IsFileUsing(id int64) bool {
	sql := `select count(*) > 0 from dfs_file where localId = ?`
	return DBUtil.SelectSingleOneIgnoreError[bool](sql, id)
}
