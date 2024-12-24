package DBUtil

import (
	"DairoDFS/dao/dto"
	"fmt"
	"reflect"
	"strconv"
	"testing"
	"time"
)

// 生成ID测试
func TestID(t *testing.T) {
	var idMap = make(map[int64]bool)
	for i := 0; i < 100; i++ {
		id := ID()
		_, isExits := idMap[id]
		if isExits {
			fmt.Printf("-->%d\n", id)
			t.Error("生成了重复的id")
		}
		idMap[id] = true
	}
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectList1(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())
	dto := SelectList[dto.UserDto]("select * from user", id)
	if dto == nil {
		t.Error("添加用户失败")
	}
	ExecIgnoreError("delete from user where id = ?", id)
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectList2(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())

	count := 10000
	var now int64

	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectListBk[dto.UserDto]("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)
	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectList[dto.UserDto]("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)
	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectToListMap("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)

	fmt.Println("--------------------------------------------------")

	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectListBk[dto.UserDto]("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)
	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectList[dto.UserDto]("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)
	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		list := SelectToListMap("select * from user")
		if list == nil {
			t.Error("添加用户失败")
		}
	}
	fmt.Println(time.Now().UnixMilli() - now)
	ExecIgnoreError("delete from user where id = ?", id)
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectList3(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())
	list := SelectList[int64]("select id from user")
	if list == nil {
		t.Error("添加用户失败")
	}
	ExecIgnoreError("delete from user where id = ?", id)
}

// 查询数据列表测试
func TestSelectOne(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, email, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?, ?)",
		id, strconv.FormatInt(id, 10), "dto.Pwd", "dto.Email", "dto.EncryptionKey", 0, 123456789)
	//list := SelectList[int64]("select urlPath from user where id = ?", id)
	//println(list)
	userDto := SelectOne[dto.UserDto]("select * from user where id = ?", id)
	//ExecIgnoreError("delete from user where id = ?", id)
	if *((*userDto).State) == 0 {
		t.Error("失败")
	}
}

func TestExecIgnoreError1(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, email, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?, ?)",
		id, strconv.FormatInt(id, 10), "dto.Pwd", "dto.Email", "dto.EncryptionKey", 0, time.Now())
	update := func() {
		ExecIgnoreError("update user date = ? where id = ?", time.Now(), id)
	}
	for i := 0; i < 100000; i++ {
		go update()
	}
	//time.Sleep(10000 * time.Hour)

	//ExecIgnoreError("delete from user where id = ?", id)
}

func TestSelectSingleOneIgnoreError(t *testing.T) {
	count := SelectSingleOneIgnoreError[int8]("select sum(id),count(*) from user")
	fmt.Println(count)
}

func TestSelectSingleOneIgnoreError2(t *testing.T) {
	name1 := SelectSingleOneIgnoreError[string]("select name from user where id = 0")
	fmt.Println(name1)

	name := SelectSingleOneIgnoreError[string]("select name from user limit 1")
	fmt.Println(name)

	apiToken := SelectSingleOneIgnoreError[string]("select apiToken from user limit 1")
	fmt.Println(apiToken)

	id := SelectSingleOneIgnoreError[string]("select id from user limit 1")
	fmt.Println(id)
}

func TestDB(t *testing.T) {
	id := ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("DtoName%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())
	user := &dto.UserDto{}

	structval := reflect.ValueOf(user).Elem()
	field := structval.FieldByName("Id")

	row := DBConn.QueryRow("select id,name from user where id = ?", id)

	scanArr := []any{
		field.Addr().Interface(),
		&user.Name,
	}
	row.Scan(scanArr...)
	fmt.Println(user.Id)
	fmt.Println(user.Name)
}
