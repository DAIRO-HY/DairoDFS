/**
 * 代码为自动生成，请勿手动修改
 */
package main

import (
	controllerapp "DairoDFS/controller/app"
	controllerappabout "DairoDFS/controller/app/about"
	controllerappfiles "DairoDFS/controller/app/files"
	controllerappinstallcreateadmin "DairoDFS/controller/app/install/create_admin"
	controllerappinstallcreateadminform "DairoDFS/controller/app/install/create_admin/form"
	controllerapplogin "DairoDFS/controller/app/login"
	controllerapploginform "DairoDFS/controller/app/login/form"
	controllerappselfset "DairoDFS/controller/app/self_set"
	controllerappuser "DairoDFS/controller/app/user"
	controllerappuserform "DairoDFS/controller/app/user/form"
	inerceptor "DairoDFS/inerceptor"
	"net/url"

	"embed"
	"encoding/json"
	"fmt"
	"github.com/go-playground/locales/zh"
	ut "github.com/go-playground/universal-translator"
	"github.com/go-playground/validator/v10"
	zh_translations "github.com/go-playground/validator/v10/translations/zh"
	"html/template"
	"io/fs"
	"log"
	"net/http"
	"strconv"
	"strings"
)

//go:embed resources/static/*
var staticFiles embed.FS

//go:embed resources/templates/*
var templatesFiles embed.FS

// 定义一个表单验证全局实例
var validate = validator.New()

var trans ut.Translator

func init() {
	zhLocale := zh.New() // 中文翻译器
	uni := ut.New(zhLocale, zhLocale)
	trans, _ = uni.GetTranslator("zh") // 注册中文翻译
	zh_translations.RegisterDefaultTranslations(validate, trans)
}

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
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerapp.Home()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/index.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerapp.Init()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/index.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/about", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappabout.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/about.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/files", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappfiles.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/files.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/install/create_admin", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerappinstallcreateadmin.Init(writer, request)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/install/create_admin.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/install/create_admin/add_admin", func(writer http.ResponseWriter, request *http.Request) {
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		inForm := controllerappinstallcreateadminform.CreateAdminForm{}
		inFormName := getStringArray(query, postForm, "name")
		if inFormName != nil { // 如果参数存在
			inForm.Name = inFormName[0]
		}

		inFormPwd := getStringArray(query, postForm, "pwd")
		if inFormPwd != nil { // 如果参数存在
			inForm.Pwd = inFormPwd[0]
		}

		validBody := validateForm(inForm)
		if validBody != nil {
			writeFieldError(writer, validBody)
			return
		}
		var body any = nil
		body = controllerappinstallcreateadmin.AddAdmin(inForm)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/login", func(writer http.ResponseWriter, request *http.Request) {
		var body any = nil
		controllerapplogin.Init(writer, request)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/login.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/login/do-login", func(writer http.ResponseWriter, request *http.Request) {
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		loginForm := controllerapploginform.LoginAppInForm{}
		loginFormName := getStringArray(query, postForm, "name")
		if loginFormName != nil { // 如果参数存在
			loginForm.Name = &loginFormName[0]
		}

		loginFormPwd := getStringArray(query, postForm, "pwd")
		if loginFormPwd != nil { // 如果参数存在
			loginForm.Pwd = &loginFormPwd[0]
		}

		loginFormDeviceId := getStringArray(query, postForm, "deviceId")
		if loginFormDeviceId != nil { // 如果参数存在
			loginForm.DeviceId = &loginFormDeviceId[0]
		}

		validBody := validateForm(loginForm)
		if validBody != nil {
			writeFieldError(writer, validBody)
			return
		}
		loginFormIsNameAndPwdMsg := loginForm.IsNameAndPwd()
		if loginFormIsNameAndPwdMsg != nil { // 表单相关验证失败
			writeFieldFormError(writer, *loginFormIsNameAndPwdMsg, "name", "pwd")
			return
		}
		var _clientFlag int // 初始化变量
		_clientFlagArr := getIntArray(query, postForm, "_clientFlag")
		if _clientFlagArr != nil { // 如果参数存在
			_clientFlag = _clientFlagArr[0]
		}
		var _version int // 初始化变量
		_versionArr := getIntArray(query, postForm, "_version")
		if _versionArr != nil { // 如果参数存在
			_version = _versionArr[0]
		}
		var body any = nil
		body = controllerapplogin.DoLogin(loginForm, _clientFlag, _version)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappselfset.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/self_set.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/self_set/init", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		body = controllerappselfset.Init()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set/make_api_token", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		var flag int // 初始化变量
		flagArr := getIntArray(query, postForm, "flag")
		if flagArr != nil { // 如果参数存在
			flag = flagArr[0]
		}
		var body any = nil
		controllerappselfset.MakeApiToken(flag)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set/make_url_path", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		var flag int // 初始化变量
		flagArr := getIntArray(query, postForm, "flag")
		if flagArr != nil { // 如果参数存在
			flag = flagArr[0]
		}
		var body any = nil
		controllerappselfset.MakeUrlPath(flag)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set/make_encryption", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		var flag int // 初始化变量
		flagArr := getIntArray(query, postForm, "flag")
		if flagArr != nil { // 如果参数存在
			flag = flagArr[0]
		}
		var body any = nil
		controllerappselfset.MakeEncryption(flag)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/user_edit", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappuser.EditHtml()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/user_edit.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/user_edit/init", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		var id int64 // 初始化变量
		idArr := getInt64Array(query, postForm, "id")
		if idArr != nil { // 如果参数存在
			id = idArr[0]
		}
		var body any = nil
		body = controllerappuser.EditInit(id)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/user_edit/edit", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		inForm := controllerappuserform.UserEditInoutForm{}
		inFormId := getInt64Array(query, postForm, "id")
		if inFormId != nil { // 如果参数存在
			inForm.Id = &inFormId[0]
		}

		inFormName := getStringArray(query, postForm, "name")
		if inFormName != nil { // 如果参数存在
			inForm.Name = &inFormName[0]
		}

		inFormEmail := getStringArray(query, postForm, "email")
		if inFormEmail != nil { // 如果参数存在
			inForm.Email = &inFormEmail[0]
		}

		inFormState := getInt8Array(query, postForm, "state")
		if inFormState != nil { // 如果参数存在
			inForm.State = &inFormState[0]
		}

		inFormDate := getStringArray(query, postForm, "date")
		if inFormDate != nil { // 如果参数存在
			inForm.Date = &inFormDate[0]
		}

		inFormPwd := getStringArray(query, postForm, "pwd")
		if inFormPwd != nil { // 如果参数存在
			inForm.Pwd = &inFormPwd[0]
		}

		filedError := map[string]*[]string{}
		isNotEmpty(filedError, "Name", inForm.Name, "")     // 非空验证
		isLength(filedError, "Name", inForm.Name, 2, 3, "") // 输入长度验证
		if len(filedError) > 0 {
			writeFieldError(writer, filedError)
			return
		}
		validBody := validateForm(inForm)
		if validBody != nil {
			writeFieldError(writer, validBody)
			return
		}
		inFormIsNameMsg := inForm.IsName()
		if inFormIsNameMsg != nil { // 表单相关验证失败
			writeFieldFormError(writer, *inFormIsNameMsg, "name")
			return
		}
		inFormIsPwdMsg := inForm.IsPwd()
		if inFormIsPwdMsg != nil { // 表单相关验证失败
			writeFieldFormError(writer, *inFormIsPwdMsg, "pwd")
			return
		}
		var body any = nil
		controllerappuser.Edit(inForm)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/user_list", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappuser.ListHtml()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		templates := append([]string{"resources/templates/app/user_list.html"}, COMMON_TEMPLATES...)
		writeToTemplate(writer, templates, body)
	})
	http.HandleFunc("/app/user_list/init", func(writer http.ResponseWriter, request *http.Request) {
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		body = controllerappuser.ListInit()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})

	// 启动服务器
	err := http.ListenAndServe(fmt.Sprintf(":%d", port), nil)
	if err != nil {
		log.Fatal(err)
	}
}

// 获取string数组类型的参数
func getStringArray(query url.Values, postForm url.Values, key string) []string {
	value, isExists := postForm[key]
	if isExists {
		return value
	}
	value, isExists = query[key]
	if isExists {
		return value
	}
	return nil
}

// 获取int数组类型的参数
func getIntArray(query url.Values, postForm url.Values, key string) []int {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]int, len(valueArray))
	for i, it := range valueArray {
		value[i], _ = strconv.Atoi(it)
	}
	return value
}

// 获取int8数组类型的参数
func getInt8Array(query url.Values, postForm url.Values, key string) []int8 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]int8, len(valueArray))
	for i, it := range valueArray {
		i8, _ := strconv.Atoi(it)
		value[i] = int8(i8)
	}
	return value
}

// 获取int16数组类型的参数
func getInt16Array(query url.Values, postForm url.Values, key string) []int16 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]int16, len(valueArray))
	for i, it := range valueArray {
		i16, _ := strconv.Atoi(it)
		value[i] = int16(i16)
	}
	return value
}

// 获取int32数组类型的参数
func getInt32Array(query url.Values, postForm url.Values, key string) []int32 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]int32, len(valueArray))
	for i, it := range valueArray {
		i32, _ := strconv.Atoi(it)
		value[i] = int32(i32)
	}
	return value
}

// 获取int64数组类型的参数
func getInt64Array(query url.Values, postForm url.Values, key string) []int64 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]int64, len(valueArray))
	for i, it := range valueArray {
		i64, _ := strconv.ParseInt(it, 10, 64)
		value[i] = i64
	}
	return value
}

// 获取float32数组类型的参数
func getFloat32Array(query url.Values, postForm url.Values, key string) []float32 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]float32, len(valueArray))
	for i, it := range valueArray {
		f32, _ := strconv.ParseFloat(it, 32)
		value[i] = float32(f32)
	}
	return value
}

// 获取float64数组类型的参数
func getFloat64Array(query url.Values, postForm url.Values, key string) []float64 {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]float64, len(valueArray))
	for i, it := range valueArray {
		f64, _ := strconv.ParseFloat(it, 64)
		value[i] = f64
	}
	return value
}

// 获取Bool数组类型的参数
func getBoolArray(query url.Values, postForm url.Values, key string) []bool {
	valueArray := getStringArray(query, postForm, key)
	if valueArray == nil {
		return nil
	}
	value := make([]bool, len(valueArray))
	for i, it := range valueArray {
		value[i] = it == "true"
	}
	return value
}

// 非空检查
func isNotEmpty(fieldError map[string]*[]string, field string, targetValue any, msg string) {
	message := "该栏必填"
	if targetValue.(any) == nil {
		addFieldErr(fieldError, field, message)
		return
	}
	value := fmt.Sprintf("%v", targetValue)
	if len(value) == 0 { //判断是否为空字符串
		addFieldErr(fieldError, field, message)
	}
}

// 输入长度检查
func isLength(fieldError map[string]*[]string, field string, targetValue any, min int, max int, msg string) {
	value := ""
	if targetValue != nil {
		value = fmt.Sprintf("%v", targetValue)
	}
	lengtn := len(value)
	message := ""
	if min > 0 && lengtn < min { //比较最小长度
		message = fmt.Sprintf("长度必须至少为%d个字符", min)
	} else if max > 0 && lengtn > max {
		message = fmt.Sprintf("长度不能超过%d个字符", max)
	} else {
	}
	addFieldErr(fieldError, field, message)
}

// 添加表单检查错误消息
func addFieldErr(fieldError map[string]*[]string, field string, message string) {
	if message != "" {
		field = strings.ToLower(field[:1]) + field[1:]
		messages, isExists := fieldError[field]
		if !isExists {
			var temp []string
			messages = &temp
			fieldError[field] = messages
		}
		*messages = append(*messages, message)
	}
}

// 表单验证
func validateForm(form any) any {
	err := validate.Struct(form)
	if err == nil {
		return nil
	}
	fieldError := map[string]*[]string{}
	for _, validErr := range err.(validator.ValidationErrors) {
		key := validErr.Field()
		key = strings.ToLower(key[:1]) + key[1:]
		messages, isExists := fieldError[key]
		if !isExists {
			var temp []string
			messages = &temp
			fieldError[key] = messages
		}
		*messages = append(*messages, validErr.Translate(trans))
	}
	body := map[string]any{
		"code": 2,
		"msg":  "参数错误",
		"data": fieldError,
	}
	return body
}

// 返回表单验证失败结果
func writeFieldError(writer http.ResponseWriter, validBody any) {

	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")
	writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
	writeToResponse(writer, validBody)
}

// 返回表单相关验证失败结果
func writeFieldFormError(writer http.ResponseWriter, msg string, fileds ...string) {

	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")
	writer.WriteHeader(http.StatusInternalServerError) // 设置状态码

	fieldError := map[string][]string{}
	for _, it := range fileds {
		fieldError[it] = []string{msg}
	}
	body := map[string]any{
		"code": 2,
		"msg":  "参数错误",
		"data": fieldError,
	}
	writeToResponse(writer, body)
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
