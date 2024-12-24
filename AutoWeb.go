/**
 * 代码为自动生成，请勿手动修改
 */
package main

import (
	controllerapp "DairoDFS/controller/app"
	controllerappinstallcreateadmin "DairoDFS/controller/app/install/create_admin"
	controllerapplogin "DairoDFS/controller/app/login"

	"embed"
	"encoding/json"
	"fmt"
	"html/template"
	"io/fs"
	"log"
	"net/http"
	"reflect"
	"strconv"
	"strings"
)

//go:embed resources/static/*
var staticFiles embed.FS

//go:embed resources/templates/*
var templatesFiles embed.FS

// 开启web服务
func startWebServer(port int) {

	// 将嵌入的资源限制到 "/resources/static" 子目录
	staticFS, staticErr := fs.Sub(staticFiles, "resources/static")
	if staticErr != nil {
		panic(staticErr)
	}

	// 使用 http.FileServer 提供文件服务
	http.Handle("/static/", http.StripPrefix("/static/", http.FileServer(http.FS(staticFS))))

	http.HandleFunc("/", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerapp.Home()
		templates := append([]string{"resources/templates/index.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerapp.Init()
		templates := append([]string{"resources/templates/index.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/install/create_admin", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerappinstallcreateadmin.Init(writer, request)
		templates := append([]string{"resources/templates/app/install/create_admin.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/login", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerapplogin.Execute()
		templates := append([]string{"resources/templates/app/login.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})

	// 启动服务器
	err := http.ListenAndServe(fmt.Sprintf(":%d", port), nil)
	if err != nil {
		log.Fatal(err)
	}
}

// 生成参数Map
func makeParamMap(request *http.Request) map[string][]string {
	query := request.URL.Query()

	//解析post表单
	request.ParseForm()
	postParams := request.PostForm

	//将参数转换成Map
	paramMap := make(map[string][]string)
	for key, v := range query {
		paramMap[key] = v
	}
	for key, v := range postParams {
		paramMap[key] = v
	}
	return paramMap
}

// 获取表单实例
func getForm[T any](paramMap map[string][]string) T {

	// 创建结构体实例
	targetForm := new(T)
	reflectForm := reflect.ValueOf(targetForm).Elem()
	argType := reflect.TypeOf(*targetForm)

	// 遍历结构体字段
	for j := 0; j < argType.NumField(); j++ {
		field := argType.Field(j)
		fieldName := field.Name

		//得到参数值
		value := paramMap[fieldName]
		if value == nil {
			//将首字母小写再去获取参数
			lowerKey := strings.ToLower(fieldName[:1]) + fieldName[1:]
			value = paramMap[lowerKey]
		}
		if value == nil {
			continue
		}

		// 设置字段值（这里我们设置为示例值）
		switch field.Type.Kind() {
		case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:

			// 设置整数字段
			intValue, _ := strconv.ParseInt(value[0], 10, 64)
			reflectForm.Field(j).SetInt(intValue)
		case reflect.Float32, reflect.Float64:
			floatValue, _ := strconv.ParseFloat(value[0], 64)
			reflectForm.Field(j).SetFloat(floatValue)
		case reflect.String:
			reflectForm.Field(j).SetString(value[0]) // 设置字符串字段
		}
	}
	return *targetForm
}

// 获取string类型的参数
func getString(paramMap map[string][]string, key string) string {
	value := paramMap[key]
	if value == nil {
		return ""
	}
	rValue := value[0]
	return rValue
}

// 获取int类型的参数
func getInt(paramMap map[string][]string, key string) int {
	value := paramMap[key]
	if value == nil {
		return 0
	}
	rValue, _ := strconv.Atoi(value[0])
	return rValue
}

// 获取int类型的参数
func getInt64(paramMap map[string][]string, key string) int64 {
	value := paramMap[key]
	if value == nil {
		return 0
	}
	rValue, _ := strconv.ParseInt(value[0], 10, 64)
	return rValue
}

// 获取float32类型的参数
func getFloat32(paramMap map[string][]string, key string) float32 {
	value := paramMap[key]
	if value == nil {
		return 0
	}
	rValue, _ := strconv.ParseFloat(value[0], 32)
	return float32(rValue)
}

// 获取float64类型的参数
func getFloat64(paramMap map[string][]string, key string) float64 {
	value := paramMap[key]
	if value == nil {
		return 0
	}
	rValue, _ := strconv.ParseFloat(value[0], 64)
	return rValue
}

// 返回结果
func writeToResponse(writer http.ResponseWriter, body any) {
	if body == nil {
		return
	}
	if body == "" {
		return
	}

	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")

	switch returnBody := body.(type) {
	case string:
		writer.Write([]uint8(returnBody))
	case int:
		writer.Write([]uint8(strconv.Itoa(returnBody)))
	case int8:
		writer.Write([]uint8(strconv.Itoa(int(returnBody))))
	case int16:
		writer.Write([]uint8(strconv.Itoa(int(returnBody))))
	case int32:
		writer.Write([]uint8(strconv.Itoa(int(returnBody))))
	case int64:
		writer.Write([]uint8(strconv.FormatInt(returnBody, 10)))
	case error:
		// 设置 HTTP 状态码
		writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
		jsonData, _ := json.Marshal(body)
		writer.Write(jsonData)
	default:
		jsonData, _ := json.Marshal(body)
		writer.Write(jsonData)
	}
}

// 写入html模板
func writeToTemplate(writer http.ResponseWriter, templates []string, data any) {

	// 解析嵌入的模板
	t, err := template.ParseFS(templatesFiles, templates...)
	if err != nil {
		http.Error(writer, fmt.Sprintf("Error loading template:%q", err), http.StatusInternalServerError)
		return
	}

	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/html;charset=UTF-8")
	t.Execute(writer, data)
}