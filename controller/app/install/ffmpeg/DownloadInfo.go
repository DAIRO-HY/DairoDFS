package ffmpeg

import "bytes"

type DownloadInfo struct {

	// 创建一个 bytes.Buffer 来保存数据到内存
	downloadBuffer bytes.Buffer

	/**
	 * 下载信息
	 */
	info string

	/**
	 * 文件总大小
	 */
	total int64

	/**
	 * 记录最后一次请求下载大小(用来计算网速)
	 */
	lastDownloadedSize int64

	/**
	 * 记录最后一次请求进度时间
	 */
	lastProgressTime int64
}
