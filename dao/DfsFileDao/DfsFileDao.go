package DfsFileDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 */
func Add(dto *dto.DfsFileDto) {
	id := DBUtil.InsertIgnoreError(`insert into dfs_file(id, userId, parentId, name, size, contentType, localId, date, isExtra, property, state)
        values (#{id}, #{userId}, #{parentId}, #{name}, #{size}, #{contentType}, #{localId}, #{date}, #{isExtra},
                #{property}, #{state})`)
	dto.Id = id
}

/**
 * 通过id获取一条数据
 * @param id 文件ID
 */
func SelectOne(id int64) *dto.DfsFileDto {
	sql := "select * from dfs_file where id = #{0}"
	return DBUtil.SelectOne[dto.DfsFileDto](sql, id)
}

/**
 * 通过文件夹ID和文件名获取文件信息
 * @param parentId 文件夹ID
 * @param name 文件名
 * @return 文件信息
 */
func SelectByParentIdAndName(userId int64, parentId int64, name string) *dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where userId = #{userId}
          and parentId = #{parentId}
          and name COLLATE NOCASE = #{name}
          and isHistory = 0
          and deleteDate is null`
	return DBUtil.SelectOne[dto.DfsFileDto](sql)
}

/**
 * 通过文件夹ID和文件名获取文件Id
 * @param parentId 文件夹ID
 * @param name 文件名
 * @return 文件信息
 */
func SelectIdByParentIdAndName(userId int64, parentId int64, name string) int64 {
	sql := `select id
        from dfs_file
        where userId = #{userId}
          and parentId = #{parentId}
          and name COLLATE NOCASE = #{name}
          and isHistory = 0
          and deleteDate is null`
	return DBUtil.SelectSingleOneIgnoreError[int64](sql)
}

/**
 * 通过路径获取文件ID
 * @param names 文件名列表
 * @return 文件ID
 */
func SelectIdByPath(userId int64, names []string) int64 {
	sql := `<foreach collection="names">
            select id from dfs_file where userId = #{userId} and parentId = (
        </foreach>
        0
        <foreach collection="names" item="name">
            ) and name COLLATE NOCASE = #{name} and isHistory = 0 and deleteDate is null
        </foreach>`
	return DBUtil.SelectSingleOneIgnoreError[int64](sql)
}

/**
 * 获取子文件id和文件名
 * @param parentId 文件夹id
 * @return 子文件列表
 */
func SelectSubFileIdAndName(userId int64, parentId int64) []*dto.DfsFileDto {
	sql := `select id, name, localId
        from dfs_file
        where userId = #{userId}
          and parentId = #{parentId}
          and isHistory = 0
          and deleteDate is null`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 获取子文件信息,客户端显示用
 * @param parentId 文件夹id
 * @return 子文件列表
 */
func SelectSubFile(userId int64, parentId int64) []*dto.DfsFileThumbDto {
	sql := `select df.id, df.name, df.size, df.date, df.localId, thumbDf.id > 0 as hasThumb
        from dfs_file as df
                 left join dfs_file as thumbDf
                           on thumbDf.parentId = df.id and df.localId > 0 and thumbDf.name = 'thumb'
        where df.userId = #{userId}
          and df.parentId = #{parentId}
          and df.isHistory = 0
          and df.deleteDate is null`
	return DBUtil.SelectList[dto.DfsFileThumbDto](sql)
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
        where df.userId = #{0}
          and df.isHistory = 0
          and df.deleteDate is not null`
	return DBUtil.SelectList[dto.DfsFileThumbDto](sql)
}

/**
 * 获取所有回收站超时的数据
 * @return 已删除的文件
 */
func SelectIdsByDeleteAndTimeout(time int64) []int64 {
	sql := `select id
        from dfs_file
        where deleteDate <![CDATA[<]]> #{0}
        limit 1000`
	return DBUtil.SelectList[int64](sql)
}

/**
 * 获取文件历史版本
 * @param userId 用户ID
 * @param id 文件id
 * @return 历史版本列表
 */
func SelectHistory(userId int64, id int64) []*dto.DfsFileDto {
	sql := `select id, size, date
        from dfs_file
        where userId = #{userId}
          and parentId = (select parentId from dfs_file where id = #{id})
          and name = (select name from dfs_file where id = #{id})
          and isHistory = 1
          and deleteDate is null`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 获取尚未处理的数据
 */
func SelectNoHandle() []*dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where localId > 0
          and state = 0
        order by id asc
        limit 1000`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 将文件标记为历史版本
 * @param id 文件ID
 */
func SetHistory(id int64) {
	sql := `update dfs_file
        set isHistory = 1
        where id = #{0}`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 将文件标记为删除
 * @param id 文件ID
 * @param time 时间戳
 */
func SetDelete(id int64, time int64) {
	sql := `update dfs_file
        set deleteDate = #{param2}
        where userId = (select userId from dfs_file where id = #{param1})
          and parentId = (select parentId from dfs_file where id = #{param1})
          and name = (select name from dfs_file where id = #{param1})
          and deleteDate is null`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 将标记为删除文件还原
 * @param id 文件ID
 */
func SetNotDelete(id int64) {
	sql := `update dfs_file
        set deleteDate = null
        where userId = (select userId from dfs_file where id = #{0})
          and parentId = (select parentId from dfs_file where id = #{0})
          and name = (select name from dfs_file where id = #{0})
          and deleteDate = (select deleteDate from dfs_file where id = #{0})`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 修改文件类型
 * @param id 文件ID
 */
func SetContentType(id int64, contentType string) {
	sql := `update dfs_file
        set contentType = #{contentType}
        where id = #{id}
          and localId > 0`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 删除
 * @param id 文件ID
 */
func Delete(id int64) {
	sql := `delete
        from dfs_file
        where id = #{0}`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 文件移动
 * @param dto 移动文件信息
 */
func Move(dto dto.DfsFileDto) {
	sql := `update dfs_file
        set parentId = #{parentId},
            name     = #{name}
        where id = #{id}`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 设置文件属性
 */
func SetProperty(id int64, property string) {
	sql := `update dfs_file
        set property = #{property}
        where id = #{id}`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 设置文件处理状态
 */
func SetState(id int64, state int8, stateMsg string) {
	sql := `update dfs_file
        set state    = #{state},
            stateMsg = #{stateMsg}
        where id = #{id}`
	DBUtil.ExecIgnoreError(sql)
}

/**
 * 验证文件存储ID权限
 */
func ValidLocalId(userId int64, localId int64) bool {
	sql := `select count(*) > 0
        from dfs_file
        where userId = #{param1}
          and localId = #{param2}`
	return DBUtil.SelectSingleOneIgnoreError[bool](sql)
}

/**
 * 获取附属文件
 * @param parentId dfs文件ID
 * @param name 附属文件标题
 * @return 附属文件信息
 */
func SelectExtra(parentId int64, name string) *dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where parentId = #{parentId}
          and name = #{name}
          and isExtra = 1`
	return DBUtil.SelectOne[dto.DfsFileDto](sql)
}

/**
 * 获取扩展文件的所有key值
 * @param id dfs文件ID
 * @return 附属文件信息
 */
func SelectExtraNames(id int64) []string {
	sql := `select name
        from dfs_file
        where parentId = #{0}
          and isExtra = 1`
	return DBUtil.SelectList[string](sql)
}

/**
 * 通过本地存储ID查询文件属性
 * @param localId 本地存储id
 * @return 属性
 */
func SelectPropertyByLocalId(localId int64) string {
	sql := `select property
        from dfs_file
        where localId = #{0}
          and state = 1
          and property is not null
        limit 1`
	return DBUtil.SelectSingleOneIgnoreError[string](sql)
}

/**
 * 通过本地存储ID查询文件附属文件
 * @param localId 本地存储id
 * @return 附属文件列表
 */
func SelectExtraFileByLocalId(localId int64) []*dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where parentId = (select id from dfs_file where localId = #{0} and state = 1 limit 1)
          and isExtra = 1`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 获取某个文件附属文件
 * @param id 文件id
 * @return 附属文件列表
 */
func SelectExtraListById(id int64) []*dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where parentId = #{0}
          and isExtra = 1`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 获取某个文件夹下的所有文件及文件夹，包括历史文件，已删除文件
 * @param id 文件id
 * @return 文件夹下的所有文件及文件夹，包括历史文件，已删除文件
 */
func SelectAllChildList(id int64) []*dto.DfsFileDto {
	sql := `select *
        from dfs_file
        where parentId = #{0}`
	return DBUtil.SelectList[dto.DfsFileDto](sql)
}

/**
 * 文件是否正在使用中
 * @param id 本地文件id
 */
func IsFileUsing(id int64) bool {
	sql := `select count(*) > 0
        from dfs_file
        where localId = #{0}`
	return DBUtil.SelectSingleOneIgnoreError[bool](sql)
}
