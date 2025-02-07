package DBUtil

import (
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Number"
	"DairoDFS/util/DBConnection"
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
		id := Number.ID()
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
	for i := 0; i < 10; i++ {
		id := Number.ID()
		InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
			id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.EncryptionKey", i, time.Now())
	}
	list := SelectList[dto.UserDto]("select *,urlPath as id2,id as id3 from user")
	for _, it := range list {
		fmt.Println(it)
	}
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectList4(t *testing.T) {
	id := Number.ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("dto.Name%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())
	SelectList[dto.UserDto]("select *,urlPath as id2,id as id3 from user")
	SelectListNull[dto.UserDto]("select *,urlPath as id2,id as id3 from user")
	ExecIgnoreError("delete from user where id = ?", id)
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectList2(t *testing.T) {
	id := Number.ID()
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
	id := Number.ID()
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
	id := Number.ID()
	InsertIgnoreError("insert into user(id, name, pwd, email, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?, ?)",
		id, strconv.FormatInt(id, 10), "dto.Pwd", "dto.Email", "dto.EncryptionKey", 0, 123456789)
	//list := SelectList[int64]("select urlPath from user where id = ?", id)
	//println(list)
	userDto, _ := SelectOne[dto.UserDto]("select * from user where id = ?", id)
	//ExecIgnoreError("delete from user where id = ?", id)
	if (userDto.State) == 0 {
		t.Error("失败")
	}
}

func TestExecIgnoreError1(t *testing.T) {
	id := Number.ID()
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
	id := Number.ID()
	InsertIgnoreError("insert into user(id, name, pwd, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?)",
		id, fmt.Sprintf("DtoName%d", id), "dto.Pwd", "dto.EncryptionKey", 1, time.Now())
	user := &dto.UserDto{}

	structval := reflect.ValueOf(user).Elem()
	field := structval.FieldByName("Id")

	row := DBConnection.DBConn.QueryRow("select id,name from user where id = ?", id)

	scanArr := []any{
		field.Addr().Interface(),
		&user.Name,
	}
	row.Scan(scanArr...)
	fmt.Println(user.Id)
	fmt.Println(user.Name)
}

// 同名文件测试
func TestUniqueName(t *testing.T) {
	//go func() {
	//	_, err := getConnection().Exec(`insert into dfs_file (id, userId, parentId, name, "size", contentType, localId, "date", property, isExtra, isHistory,
	//                  deleteDate, state, stateMsg)values (?,1,1,'abc',1,'text',0,0,null,0,0,null,0,null);`, Number.ID())
	//	fmt.Printf("1 -> %d\n", time.Now().UnixMilli())
	//	time.Sleep(3 * time.Second)
	//	Commit()
	//	if err != nil {
	//		fmt.Println(err)
	//	}
	//}()
	//go func() {
	//	_, err := getConnection().Exec(`insert into dfs_file (id, userId, parentId, name, "size", contentType, localId, "date", property, isExtra, isHistory,
	//                  deleteDate, state, stateMsg)values (?,1,1,'abc',1,'text',0,0,null,0,0,null,0,null);`, Number.ID())
	//	fmt.Printf("2 -> %d\n", time.Now().UnixMilli())
	//	time.Sleep(3 * time.Second)
	//	Commit()
	//	if err != nil {
	//		fmt.Println(err)
	//	}
	//}()
	//for i := 0; i < 6; i++ {
	//	go func() {
	//		rows, _ := DBConn.Query(`select *  from main.dfs_file limit 1`, Number.ID())
	//		fmt.Printf("-->%d\n", i)
	//		if rows != nil {
	//			rows.Close()
	//		}
	//	}()
	//}
	//for i := 0; i < 5; i++ {
	//	go func() {
	//		row := DBConn.QueryRow(`select id,name  from main.dfs_file limit 1`, Number.ID())
	//		fmt.Printf("-->%d\n", i)
	//		if row != nil {
	//			var v int64
	//			row.Scan(&v)
	//		}
	//	}()
	//}
	//for i := 0; i < 5; i++ {
	//	go func() {
	//		row := getConnection().QueryRow(`select id,name  from main.dfs_file limit 1`, Number.ID())
	//		fmt.Printf("-->%d\n", i)
	//		if row != nil {
	//			//var v int64
	//			//row.Scan(&v)
	//		}
	//		Rollback()
	//	}()
	//}

	//for i := 0; i < 10; i++ {
	//	go func() {
	//		tx := getConnection()
	//		fmt.Printf("%d:tx := getConnection()\n", i)
	//		_, err := tx.Exec(`insert into dfs_file (id, userId, parentId, name, "size", contentType, localId, "date", property, isExtra, isHistory,
	//		                 deleteDate, state, stateMsg)values (?,1,?,'abc',1,'text',0,0,null,0,0,null,0,null);`, Number.ID(), Number.ID())
	//		fmt.Printf("%d -> %d\n", i, time.Now().UnixMilli())
	//		time.Sleep(1 * time.Second)
	//		//Commit()
	//		if err != nil {
	//			fmt.Printf("%d-err:%q\n", i, err)
	//		}
	//	}()
	//}
	//time.Sleep(10 * time.Millisecond)
	//go func() {
	//	var count int
	//	tx := getConnection()
	//	fmt.Println("tx := getConnection()")
	//	row := tx.QueryRow(`select count(*) from dfs_file limit 1`)
	//	row.Scan(&count)
	//	fmt.Printf("count-->%d\n", count)
	//}()
	//time.Sleep(10 * time.Millisecond)
	//for {
	//
	//	// 获取连接池统计信息
	//	//stats := DBConn.Stats()
	//	//fmt.Printf("当前打开的连接数: %d      ", stats.OpenConnections)
	//	//fmt.Printf("正在使用的连接数: %d      ", stats.InUse)
	//	//fmt.Printf("空闲的连接数: %d\n", stats.Idle)
	//	time.Sleep(3000 * time.Millisecond)
	//}

	DBConnection.StartTransaction()
	DBConnection.Write(`insert into dfs_file (id, userId, parentId, name, "size", contentType, localId, "date", property, isExtra, isHistory,
			                 deleteDate, state, stateMsg)values (?,1,?,'abc',1,'text',0,0,null,0,0,null,0,null);`, Number.ID(), Number.ID())
	DBConnection.Commit()
	var count int
	DBConnection.QueryRow("select count(*) from main.dfs_file").Scan(&count)
	fmt.Println(count)
	//Commit()
}

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func TestSelectOne2(t *testing.T) {
	count, _ := SelectSingleOne[int64]("select pwd from user")
	fmt.Println(count)
}
