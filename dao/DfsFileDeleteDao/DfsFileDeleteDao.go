package DfsFileDeleteDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 */
func Insert(id int64) {
	sql := `insert into dfs_file_delete
        select *
        from dfs_file
        where id = #{0}`
	DBUtil.InsertIgnoreError(sql)
}

/**
 * 设置删除时间
 * @param id 文件ID
 * @param time 时间戳
 */
func SetDeleteDate(id int64, time int64) {
	DBUtil.ExecIgnoreError(`update dfs_file_delete
        set deleteDate = #{param2}
        where id = #{param1}`)
}

/**
 * 获取所有超时的数据
 * @param time 时间戳
 */
func SelectIdsByTimeout(time int64) []*dto.DfsFileDto {
	return DBUtil.SelectList[dto.DfsFileDto](`select *
        from dfs_file_delete
        where deleteDate <![CDATA[<]]> #{0}
        limit 1000`)
}

/**
 * 文件是否正在使用中
 * @param id 本地文件id
 */
func IsFileUsing(id int64) bool {
	return DBUtil.SelectSingleOneIgnoreError[bool](`
		select count(*) > 0
        from dfs_file_delete
        where localId = #{0}`)
}
