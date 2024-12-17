package DBUtil

import (
	"DairoDFS/util/CommonUtil"
	"DairoDFS/util/LogUtil"
	"database/sql"
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	"log"
	"reflect"
	"strings"
	"sync"
	"time"
)

// DB_PATH 文件路径
const DB_PATH = "./data/dairo-dfs.sqlite"

var makeIdLock sync.Mutex

// sqlite数据库连接对象
var DBConn *sql.DB

func init() {
	db, err := sql.Open("sqlite3", DB_PATH)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("打开数据库失败 err:%q", err))
		log.Fatal(err)
	}
	DBConn = db
}

/**
 * 生成数据库主键ID
 */
func ID() int64 {
	makeIdLock.Lock()

	//这里延迟1纳秒，降低生成ID的重复概率
	time.Sleep(1 * time.Microsecond)
	id := time.Now().UnixMicro()
	makeIdLock.Unlock()
	return id
}

// 执行sql语句,忽略错误
func ExecIgnoreError(query string, args ...any) int64 {
	count, err := Exec(query, args...)
	if err != nil {
		log.Printf("%q: %s\n", err, query)
		return -1
	}
	return count
}

// 执行sql
func Exec(query string, args ...any) (int64, error) {
	rs, err := DBConn.Exec(query, args...)
	if err != nil {
		return -1, err
	}
	count, err := rs.RowsAffected()
	if err != nil {
		return -1, err
	}
	return count, nil
}

// 添加数据,忽略错误
func InsertIgnoreError(query string, args ...any) int64 {
	count, err := Insert(query, args...)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("添加数据失败:%s  err:%q\n", query, err))
		return -1
	}
	return count
}

// 添加数据,并返回最后一次添加的ID
func Insert(insert string, args ...any) (int64, error) {
	rs, err := DBConn.Exec(insert, args...)
	if err != nil {
		return -1, err
	}
	lastInsertId, err := rs.LastInsertId()
	if err != nil {
		return -1, err
	}
	return lastInsertId, nil
}

// SelectSingleOneIgnoreError 查询第一个数据并忽略错误
func SelectSingleOneIgnoreError[T any](query string, args ...any) *T {
	value, _ := SelectSingleOne[T](query, args...)
	return value
}

// SelectSingleOne 查询第一个数据
func SelectSingleOne[T any](query string, args ...any) (*T, error) {
	row := DBConn.QueryRow(query, args...)
	var value T
	err := row.Scan(&value) // 使用 Scan 将结果赋值给 value
	if err != nil {
		LogUtil.Error(fmt.Sprintf("error: %q, sql: %s", err, query))
		return nil, err // 返回默认值和错误
	}
	return &value, nil
}

// SelectOne 查询第一个数据
func SelectOne[T any](query string, args ...any) *T {
	dtoList := SelectList[T](query, args...)
	if len(dtoList) == 0 {
		return nil
	}
	return dtoList[0]
}

// SelectList 查询列表
func SelectListBk[T any](query string, args ...any) []*T {
	list := SelectToListMap(query, args...)

	// 创建一个空切片
	dtoList := make([]*T, 0) // 初始化空切片
	for _, item := range list {
		dtoT := new(T)
		reflectDto := reflect.ValueOf(dtoT).Elem()
		for key := range item { //遍历查询到的数据
			value := item[key]
			if value == nil { //该值为空
				continue
			}

			//结构体中的变量名
			varName := strings.ToUpper(string(key[0])) + key[1:]
			field := reflectDto.FieldByName(varName)
			if !field.IsValid() { //结构体中不存在该变量
				continue
			}
			setValue(field, value)
		}
		dtoList = append(dtoList, dtoT)
	}
	return dtoList
}

// SelectList 查询列表
func SelectList[T any](query string, args ...any) []*T {
	rows, err := DBConn.Query(query, args...)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("查询数据失败:%s: err:%q", query, err))
		return nil
	}
	defer rows.Close()

	// 获取列的名称
	columns, err := rows.Columns()
	if err != nil {
		LogUtil.Error(fmt.Sprintf("%q: %s\n", err, query))
		return nil
	}

	// 创建一个[]interface{}的slice, 每个元素指向values中的对应位置
	valuePtrs := make([]any, len(columns))

	// 创建一个空切片
	dtoList := make([]*T, 0) // 初始化空切片

	for rows.Next() {
		dtoT := new(T)
		if CommonUtil.IsBaseType(*dtoT) { //这是一个基本数据类型
			if scanErr := rows.Scan(dtoT); scanErr != nil {
				LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
				return nil
			}
			dtoList = append(dtoList, dtoT)
			continue
		}
		reflectDto := reflect.ValueOf(dtoT).Elem()
		for i, column := range columns {

			//结构体中的变量名
			varName := strings.ToUpper(string(column[0])) + column[1:]
			field := reflectDto.FieldByName(varName)
			if !field.IsValid() { //结构体中不存在该变量
				var temp any
				temp = nil
				valuePtrs[i] = &temp
			} else {
				valuePtrs[i] = field.Addr().Interface()
			}
		}

		// 将当前行的数据扫描到valuePtrs中
		if scanErr := rows.Scan(valuePtrs...); scanErr != nil {
			LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
			return nil
		}
		dtoList = append(dtoList, dtoT)
	}
	return dtoList
}

// SelectToListMap 将查询结果以List<Map>的类型返回
func SelectToListMap(query string, args ...any) []map[string]any {
	rows, err := DBConn.Query(query, args...)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("查询数据失败:%s: err:%q", query, err))
		return nil
	}
	defer rows.Close()

	// 获取列的名称
	columns, err := rows.Columns()
	if err != nil {
		LogUtil.Error(fmt.Sprintf("%q: %s\n", err, query))
		return nil
	}

	// 创建一个[]interface{}的slice, 每个元素指向values中的对应位置
	valuePtrs := make([]interface{}, len(columns))

	// 创建一个空切片
	list := make([]map[string]any, 0) // 初始化空切片
	for rows.Next() {

		// 创建一个长度与列数相同的slice来存放查询结果
		values := make([]interface{}, len(columns))
		for i := range values {
			valuePtrs[i] = &values[i]
		}

		// 将当前行的数据扫描到valuePtrs中
		if err := rows.Scan(valuePtrs...); err != nil {
			LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, err))
			return nil
		}

		// 使用map将列名和对应的值关联起来
		rowMap := make(map[string]any)
		for i, col := range columns {
			value := values[i]
			rowMap[col] = value
		}
		list = append(list, rowMap)
	}
	return list
}

// 通过反射的方式给结构体设置值
func setValue(field reflect.Value, value any) {
	kind := field.Kind()
	if kind == reflect.Ptr { //设置指针变量值
		ptr := reflect.New(field.Type().Elem())
		kindStr := field.Type().String()
		switch kindStr {
		case "*int32":
			ptr.Elem().Set(reflect.ValueOf(int32(value.(int64))))
		case "*int16":
			ptr.Elem().Set(reflect.ValueOf(int16(value.(int64))))
		case "*int8":
			ptr.Elem().Set(reflect.ValueOf(int8(value.(int64))))
		case "*int":
			ptr.Elem().Set(reflect.ValueOf(int32(value.(int64))))
		case "*float32":
			ptr.Elem().Set(reflect.ValueOf(float32(value.(float64))))
		default:
			ptr.Elem().Set(reflect.ValueOf(value))
		}
		field.Set(ptr)
	} else if kind == reflect.Int || kind == reflect.Int8 || kind == reflect.Int16 || kind == reflect.Int32 || kind == reflect.Int64 {
		field.SetInt(value.(int64))
	} else if kind == reflect.Float32 || kind == reflect.Float64 {
		field.SetFloat(value.(float64))
	} else {
		field.Set(reflect.ValueOf(value))
	}
}
