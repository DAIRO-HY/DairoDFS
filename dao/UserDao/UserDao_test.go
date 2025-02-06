package UserDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
	"fmt"
	"testing"
	"time"
)

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestAdd(t *testing.T) {

	name := "1000"
	id := Number.ID()
	state := int8(1)
	date := time.Now()
	insertDto := dto.UserDto{
		Name:  &name,
		Id:    &id,
		State: &state,
		Date:  &date,
	}
	Add(insertDto)
	count := DBUtil.SelectSingleOneIgnoreError[int64]("select count(*) from user where id = ?", insertDto.Id)
	if count != 1 {
		t.Error("添加用户失败")
	}
	DBUtil.ExecIgnoreError("delete from user where id = ?", insertDto.Id)
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectOne(t *testing.T) {
	id := Number.ID()
	DBUtil.InsertIgnoreError("insert into user(id, name, pwd, email, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.Email", "dto.EncryptionKey", 1, time.Now())
	dto := SelectOne(id)
	if dto == nil {
		t.Error("添加用户失败")
	}
	DBUtil.ExecIgnoreError("delete from user where id = ?", id)
}
