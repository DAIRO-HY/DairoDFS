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
	controllerappmodifypwd "DairoDFS/controller/app/modify_pwd"
	controllerappmodifypwdform "DairoDFS/controller/app/modify_pwd/form"
	controllerappprofile "DairoDFS/controller/app/profile"
	controllerappprofileform "DairoDFS/controller/app/profile/form"
	controllerappselfset "DairoDFS/controller/app/self_set"
	controllerappuser "DairoDFS/controller/app/user"
	controllerappuserform "DairoDFS/controller/app/user/form"
	inerceptor "DairoDFS/inerceptor"

	"embed"
	"encoding/json"
	"fmt"
	"html/template"
	"io/fs"
	"log"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"
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
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerapp.Home()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerapp.Init()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/about.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappabout.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/about.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html")
	})
	http.HandleFunc("/app/files.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappfiles.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/files.html", "resources/templates/app/include/files_list.html", "resources/templates/app/include/files/files_right_option.html", "resources/templates/app/include/files/files_share.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html", "resources/templates/app/include/files/files_toolbar.html", "resources/templates/app/include/files/files_upload.html")
	})
	http.HandleFunc("/app/install/create_admin", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		var body any = nil
		controllerappinstallcreateadmin.Init(writer, request)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/install/create_admin.html", "resources/templates/app/include/head.html")
	})
	http.HandleFunc("/app/install/create_admin/add_admin", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm
		inForm:=controllerappinstallcreateadminform.CreateAdminForm{}
		inFormName := getStringArray(query,postForm,"name")
		if inFormName != nil {// 如果参数存在
			inForm.Name = inFormName[0]
		}

		inFormPwd := getStringArray(query,postForm,"pwd")
		if inFormPwd != nil {// 如果参数存在
			inForm.Pwd = inFormPwd[0]
		}

		var body any = nil
		body = controllerappinstallcreateadmin.AddAdmin(inForm)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/login.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		var body any = nil
		controllerapplogin.Init(writer, request)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/login.html", "resources/templates/app/include/head.html")
	})
	http.HandleFunc("/app/login/do-login", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm

		// 记录表单验证错误信息
		filedError := map[string][]string{}
		validName := getStringArray(query, postForm, "name")
		isNotEmpty(filedError, "name", validName) // 非空验证
		isLength(filedError, "name", validName, 2, 32)// 输入长度验证
		validPwd := getStringArray(query, postForm, "pwd")
		isNotEmpty(filedError, "pwd", validPwd) // 非空验证
		isLength(filedError, "pwd", validPwd, 2, 32)// 输入长度验证
		validDeviceId := getStringArray(query, postForm, "deviceId")
		isNotEmpty(filedError, "deviceId", validDeviceId) // 非空验证
		if len(filedError) > 0{ // 有表单验证错误信息
			writeFieldError(writer, filedError)
			return
		}

		loginForm:=controllerapploginform.LoginAppInForm{}
		loginFormName := getStringArray(query,postForm,"name")
		if loginFormName != nil {// 如果参数存在
			loginForm.Name = loginFormName[0]
		}

		loginFormPwd := getStringArray(query,postForm,"pwd")
		if loginFormPwd != nil {// 如果参数存在
			loginForm.Pwd = loginFormPwd[0]
		}

		loginFormDeviceId := getStringArray(query,postForm,"deviceId")
		if loginFormDeviceId != nil {// 如果参数存在
			loginForm.DeviceId = loginFormDeviceId[0]
		}

		loginFormIsNameAndPwdMsg := loginForm.IsNameAndPwd()
		if loginFormIsNameAndPwdMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, loginFormIsNameAndPwdMsg, "name", "pwd")
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
	http.HandleFunc("/app/login/logout", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		var body any = nil
		controllerapplogin.Logout(request)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/modify_pwd.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappmodifypwd.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/modify_pwd.html", "resources/templates/app/include/head.html")
	})
	http.HandleFunc("/app/modify_pwd/modify", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm

		// 记录表单验证错误信息
		filedError := map[string][]string{}
		validOldPwd := getStringArray(query, postForm, "oldPwd")
		isNotBlank(filedError, "oldPwd", validOldPwd) // 非空白验证
		isLength(filedError, "oldPwd", validOldPwd, 4, 32)// 输入长度验证
		validPwd := getStringArray(query, postForm, "pwd")
		isNotBlank(filedError, "pwd", validPwd) // 非空白验证
		isLength(filedError, "pwd", validPwd, 4, 32)// 输入长度验证
		if len(filedError) > 0{ // 有表单验证错误信息
			writeFieldError(writer, filedError)
			return
		}

		inForm:=controllerappmodifypwdform.ModifyPwdAppForm{}
		inFormOldPwd := getStringArray(query,postForm,"oldPwd")
		if inFormOldPwd != nil {// 如果参数存在
			inForm.OldPwd = inFormOldPwd[0]
		}

		inFormPwd := getStringArray(query,postForm,"pwd")
		if inFormPwd != nil {// 如果参数存在
			inForm.Pwd = inFormPwd[0]
		}

		inFormIsOldPwdMsg := inForm.IsOldPwd()
		if inFormIsOldPwdMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, inFormIsOldPwdMsg, "oldPwd")
			return
		}
		var body any = nil
		body = controllerappmodifypwd.Modify(inForm)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/profile.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappprofile.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/profile.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html")
	})
	http.HandleFunc("/app/profile/init", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		body = controllerappprofile.Init()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/profile/update", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm

		// 记录表单验证错误信息
		filedError := map[string][]string{}
		validUploadMaxSize := getStringArray(query, postForm, "uploadMaxSize")
		isDigits(filedError, "uploadMaxSize", validUploadMaxSize, 11, 0)// 数值值区间验证
		isNotBlank(filedError, "uploadMaxSize", validUploadMaxSize) // 非空白验证
		validFolders := getStringArray(query, postForm, "folders")
		isNotBlank(filedError, "folders", validFolders) // 非空白验证
		if len(filedError) > 0{ // 有表单验证错误信息
			writeFieldError(writer, filedError)
			return
		}

		form:=controllerappprofileform.ProfileForm{}
		formOpenSqlLog := getBoolArray(query,postForm,"openSqlLog")
		if formOpenSqlLog != nil {// 如果参数存在
			form.OpenSqlLog = formOpenSqlLog[0]
		}

		formHasReadOnly := getBoolArray(query,postForm,"hasReadOnly")
		if formHasReadOnly != nil {// 如果参数存在
			form.HasReadOnly = formHasReadOnly[0]
		}

		formUploadMaxSize := getInt64Array(query,postForm,"uploadMaxSize")
		if formUploadMaxSize != nil {// 如果参数存在
			form.UploadMaxSize = formUploadMaxSize[0]
		}

		formFolders := getStringArray(query,postForm,"folders")
		if formFolders != nil {// 如果参数存在
			form.Folders = formFolders[0]
		}

		formSyncDomains := getStringArray(query,postForm,"syncDomains")
		if formSyncDomains != nil {// 如果参数存在
			form.SyncDomains = formSyncDomains[0]
		}

		formToken := getStringArray(query,postForm,"token")
		if formToken != nil {// 如果参数存在
			form.Token = formToken[0]
		}

		formIsFoldersMsg := form.IsFolders()
		if formIsFoldersMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, formIsFoldersMsg, "folders")
			return
		}
		var body any = nil
		body = controllerappprofile.Update(form)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/profile/app/profile/make_token", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappprofile.MakeToken()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappselfset.Html()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/self_set.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html")
	})
	http.HandleFunc("/app/self_set/init", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		body = controllerappselfset.Init()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/self_set/make_api_token", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
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
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
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
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
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
	http.HandleFunc("/app/user_edit.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappuser.EditHtml()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/user_edit.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html")
	})
	http.HandleFunc("/app/user_edit/init", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
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
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		query := request.URL.Query()
		//解析post表单
		request.ParseForm()
		postForm := request.PostForm

		// 记录表单验证错误信息
		filedError := map[string][]string{}
		validName := getStringArray(query, postForm, "name")
		isNotEmpty(filedError, "name", validName) // 非空验证
		isLength(filedError, "name", validName, 2, 32)// 输入长度验证
		validEmail := getStringArray(query, postForm, "email")
		isEmail(filedError, "email", validEmail) // 邮箱格式验证
		if len(filedError) > 0{ // 有表单验证错误信息
			writeFieldError(writer, filedError)
			return
		}

		inForm:=controllerappuserform.UserEditInoutForm{}
		inFormId := getInt64Array(query,postForm,"id")
		if inFormId != nil {// 如果参数存在
			inForm.Id = inFormId[0]
		}

		inFormName := getStringArray(query,postForm,"name")
		if inFormName != nil {// 如果参数存在
			inForm.Name = inFormName[0]
		}

		inFormEmail := getStringArray(query,postForm,"email")
		if inFormEmail != nil {// 如果参数存在
			inForm.Email = inFormEmail[0]
		}

		inFormState := getInt8Array(query,postForm,"state")
		if inFormState != nil {// 如果参数存在
			inForm.State = inFormState[0]
		}

		inFormDate := getStringArray(query,postForm,"date")
		if inFormDate != nil {// 如果参数存在
			inForm.Date = inFormDate[0]
		}

		inFormPwd := getStringArray(query,postForm,"pwd")
		if inFormPwd != nil {// 如果参数存在
			inForm.Pwd = inFormPwd[0]
		}

		inFormIsNameMsg := inForm.IsName()
		if inFormIsNameMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, inFormIsNameMsg, "name")
			return
		}
		inFormIsPwdMsg := inForm.IsPwd()
		if inFormIsPwdMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, inFormIsPwdMsg, "pwd")
			return
		}
		inFormIsEmailMsg := inForm.IsEmail()
		if inFormIsEmailMsg != "" { // 表单相关验证失败
			writeFieldFormError(writer, inFormIsEmailMsg, "email")
			return
		}
		var body any = nil
		controllerappuser.Edit(inForm)
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToResponse(writer, body)
	})
	http.HandleFunc("/app/user_list.html", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "GET" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
		if !inerceptor.LoginValidate(writer, request) {
			return
		}
		var body any = nil
		controllerappuser.ListHtml()
		body = inerceptor.RemoveGoroutineLocal(writer, request, body)
		writeToTemplate(writer, body, "resources/templates/app/user_list.html", "resources/templates/app/include/head.html", "resources/templates/app/include/top-bar.html")
	})
	http.HandleFunc("/app/user_list/init", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != "POST" {
			writer.WriteHeader(http.StatusMethodNotAllowed) // 设置状态码
			writer.Write([]byte("Method Not Allowed"))
			return
		}
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

// 非空字符检查
func isNotEmpty(fieldError map[string][]string, field string, value []string) {
	message := "不能为空"
	if value == nil {
		addFieldErr(fieldError, field, message)
		return
	}
	if len(value[0]) == 0 { //判断是否为空字符串
		addFieldErr(fieldError, field, message)
		return
	}
}

// 非空白字符检查
func isNotBlank(fieldError map[string][]string, field string, value []string) {
	message := "不能为空白"
	if value == nil {
		addFieldErr(fieldError, field, message)
		return
	}
	if len(strings.TrimSpace(value[0])) == 0 { //判断是否为空字符串
		addFieldErr(fieldError, field, message)
		return
	}
}

// 输入长度检查
func isLength(fieldError map[string][]string, field string, value []string, min int, max int) {

	//字符个数
	length := 0
	if value != nil {
		length = utf8.RuneCountInString(value[0])
	}
	if min != -1 && max != -1 {
		if length < min || length > max {
			message := fmt.Sprintf("长度必须在%d～%d个字符之间", min, max)
			addFieldErr(fieldError, field, message)
		}
		return
	}
	if min != -1 && length < min { //比较最小长度
		message := fmt.Sprintf("长度至少输入%d个字符", min)
		addFieldErr(fieldError, field, message)
		return
	}
	if max != -1 && length > max { //比较最大长度
		message := fmt.Sprintf("长度不能超过%d个字符", max)
		addFieldErr(fieldError, field, message)
	}
}

// 数值大小检查
func isLimit(fieldError map[string][]string, field string, value []string, min *float64, max *float64) {
	if value == nil { //不需要验证空
		return
	}
	if value[0] == "" {
		return
	}
	floatValue, err := strconv.ParseFloat(value[0], 64)
	if err != nil {
		addFieldErr(fieldError, field, "这不是一个正确的数值")
	}
	if min != nil && max != nil {
		if floatValue < *min || floatValue > *max {
			message := fmt.Sprintf("输入的值必须在%s～%s之间", floatToStr(*min), floatToStr(*max))
			addFieldErr(fieldError, field, message)
		}
		return
	}
	if min != nil && floatValue < *min { //比较最小长度
		message := fmt.Sprintf("输入的值不能小于%f", *min)
		addFieldErr(fieldError, field, message)
		return
	}
	if max != nil && floatValue > *max {
		message := fmt.Sprintf("输入的值不能大于%f", *max)
		addFieldErr(fieldError, field, message)
	}
}

// 数值检查
func isDigits(fieldError map[string][]string, field string, value []string, integer int, fraction int) {
	if value == nil { //不需要验证空
		return
	}
	if value[0] == "" {
		return
	}

	//点所在的位置
	dotIndex := strings.Index(value[0], ".")
	var integerStr string  //整数部分的字符串
	var fractionStr string //小数部分的字符串
	if dotIndex != -1 {
		integerStr = value[0][:dotIndex]
		fractionStr = value[0][dotIndex+1:]
	} else {
		integerStr = value[0]
	}
	for _, it := range integerStr {
		if !unicode.IsDigit(it) {
			addFieldErr(fieldError, field, "只能输入数值")
			return
		}
	}
	for _, it := range fractionStr {
		if !unicode.IsDigit(it) {
			addFieldErr(fieldError, field, "只能输入数值")
			return
		}
	}
	message := fmt.Sprintf("整数不能超过%d位", integer)
	if fraction > 0 {
		message += fmt.Sprintf("，且小数不能超过%d位", fraction)
	}
	if integer > 0 && len(integerStr) > integer { //超出了整数位数
		addFieldErr(fieldError, field, message)
		return
	}
	if fraction > 0 && len(fractionStr) > fraction { //超出了小数位数
		addFieldErr(fieldError, field, message)
		return
	}
}

// 半角检查
// - upper 是否允许大写字母
// - lower 是否允许小写字母
// - number 是否允许数字
// - symbol 是否允许符号
func isHalf(fieldError map[string][]string, field string, value []string, upper bool, lower bool, number bool, symbol bool) {
	if value == nil { //不需要验证空
		return
	}
	if value[0] == "" {
		return
	}
	message := "只能是半角"
	if upper {
		message += "大写字母、"
	}
	if lower {
		message += "小写字母、"
	}
	if number {
		message += "数字、"
	}
	if symbol {
		message += "符号、"
	}
	if !strings.HasSuffix(message, "、") { //如果结尾不是顿号，说明不允许输入任何半角字符
		addFieldErr(fieldError, field, "配置错误，至少允许输入一种半角字符")
		return
	}
	message = message[0:strings.LastIndex(message, "、")] //去掉最后一个标点符号(一个汉字占3个字节)
	for _, it := range value[0] {
		if it < 33 || it > 126 || it == 94 || it == 124 { //非可见字符
			addFieldErr(fieldError, field, message)
			return
		}
		if !upper && it >= 65 && it <= 90 { //不允许大写字母
			addFieldErr(fieldError, field, message)
			return
		}
		if !lower && it >= 97 && it <= 122 { //不允许小写字母
			addFieldErr(fieldError, field, message)
			return
		}
		if !number && it >= 48 && it <= 57 { //不允许大写字母
			addFieldErr(fieldError, field, message)
			return
		}
		if !symbol && ((it >= 33 && it <= 47) || (it >= 58 && it <= 64) || (it >= 91 && it <= 96) || (it >= 123 && it <= 126)) { //不允许特殊字符
			addFieldErr(fieldError, field, message)
			return
		}
	}
}

// 是否邮箱地址判断
func isEmail(fieldError map[string][]string, field string, value []string) {
	if value == nil { //不需要验证空
		return
	}
	if value[0] == "" {
		return
	}
	message := "请输入一个正确的邮箱地址"

	// 这是一个简单的邮箱验证表达式
	regex := `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`
	r := regexp.MustCompile(regex)
	if !r.MatchString(value[0]) {
		addFieldErr(fieldError, field, message)
	}
}

// 浮点型转字符串,去掉后面的0
func floatToStr(f float64) string {
	result := strconv.FormatFloat(f, 'f', 6, 64)
	for i := len(result) - 1; i >= 0; i-- {
		if result[i] == 46 {
			return result[0:i]
		}
		if result[i] != 48 {
			return result[0 : i+1]
		}
	}
	return "0"
}

// 添加表单检查错误消息
func addFieldErr(fieldError map[string][]string, field string, message string) {
	field = strings.ToLower(field[:1]) + field[1:]
	_, isExist := fieldError[field]
	if !isExist {
		fieldError[field] = []string{}
	}
	fieldError[field] = append(fieldError[field], message)
}

// 返回表单验证失败结果
func writeFieldError(writer http.ResponseWriter, fieldError map[string][]string) {

	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")
	writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
	validBody := map[string]any{
		"code": 2,
		"msg":  "参数错误",
		"data": fieldError,
	}
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
func writeToTemplate(writer http.ResponseWriter, data any, templates ...string) {

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

// 返回一个int类型的指针
func intP(i int) *int {
	return &i
}

// 返回一个float64类型的指针
func floatP(f float64) *float64 {
	return &f
}
