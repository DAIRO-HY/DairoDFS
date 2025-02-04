package sync

import (
	"io"
	"net"
	"net/http"
	"time"
)

/**
 * 同步数据专用的HTTP请求工具类
 */

/**
 * 请求同步数据
 * @param url 请求url
 * @return 返回结果
 */
func request(url string) string {
	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: 30 * time.Second,                                    //读数据超时
	}
	client := &http.Client{Transport: transport}

	// 创建HTTP GET请求
	resp, err := client.Get(url)
	if err != nil {
		//mine.Info = fmt.Sprintf("下载失败：%q", err)
		return ""
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		//@TODO:应该返回错误信息
		return ""
	}
	data, _ := io.ReadAll(resp.Body)
	return string(data)
}
