package LocalFileDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 */
func Add(dto dto.LocalFileDto) {
	DBUtil.InsertIgnoreError("insert into local_file(id, path, md5) values (?,?,?)", dto.Id, dto.Path, dto.Md5)
}

/**
 * 通过id获取一条数据
 * @param id 文件ID
 */
func SelectOne(id int64) *dto.LocalFileDto {
	return DBUtil.SelectOne[dto.LocalFileDto](`select * from local_file where id = ?`, id)
}

/**
 * 通过文件MD5获取一条数据
 * @param md5 文件MD5
 */
func SelectByFileMd5(md5 string) *dto.LocalFileDto {
	return DBUtil.SelectOne[dto.LocalFileDto](`select * from local_file where md5 = ?`, md5)
}
