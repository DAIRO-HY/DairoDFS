package UserDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
	"testing"
)

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestAdd(t *testing.T) {
	insertDto := dto.UserDto{
		Name:  "1000",
		Id:    DBUtil.ID(),
		State: 0,
		Date:  123456789,
	}
	Add(insertDto)
	db := DBUtil.GetDb()
	defer db.Close()
	count := DBUtil.SelectSingleOneIgnoreError[int64]("select count(*) from user where id = ?", insertDto.Id)
	if count != 1 {
		t.Error("添加用户失败")
	}
	DBUtil.ExecIgnoreError("delete from user where id = ?", insertDto.Id)
}
