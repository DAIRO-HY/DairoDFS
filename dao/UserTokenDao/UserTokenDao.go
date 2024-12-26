package UserTokenDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func Add(dto dto.UserTokenDto) {
	DBUtil.InsertIgnoreError(`insert into user_token(id, token, userId, clientFlag, deviceId, ip, date, version)
        values (?,?,?,?,?,?,?,?)`, dto.Id, dto.Token, dto.UserId, dto.ClientFlag, dto.DeviceId, dto.Ip, dto.Date, dto.Version)
}

/**
 * 通过登录Token获取会员ID
 * @param token 登录Token
 */
func GetByUserIdByToken(token string) *int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64]("select userId from user_token where token = ?", token)
}

/**
 * 获取某个用户的登录记录
 * @param userId 用户ID
 */
func ListByUserId(userId int64) []*dto.UserTokenDto {
	return DBUtil.SelectList[dto.UserTokenDto]("select * from user_token where userId = ? order by date", userId)
}

/**
 * 更新会员登录记录
 * @param dto 用户信息
 */
func Update(dto dto.UserTokenDto) {
	DBUtil.ExecIgnoreError(`update user_token set date = ?, version = ?, ip = ? where token = ?`,
		dto.Date, dto.Version, dto.Ip, dto.Token)
}

/**
 * 通过会员ID和客户端标识删除一条记录
 * @param userId 用户ID
 * @param clientFlag 客户端标志
 */
func DeleteByUserIdAndClientFlag(userId int64, clientFlag int) {
	DBUtil.ExecIgnoreError("delete from user_token where userId = ? and clientFlag = ?", userId, clientFlag)
}

/**
 * 通过会员ID和客户端标识删除一条记录
 * @param userId 用户ID
 * @param deviceId 设备唯一标识
 */
func DeleteByUserIdAndDeviceId(userId int64, deviceId string) {
	DBUtil.ExecIgnoreError(`delete from user_token where userId = ? and deviceId = ?`, userId, deviceId)
}

/**
 * 删除某个会员的所有登录token
 * @param userId 用户ID
 */
func DeleteByUserId(userId int64) {
	DBUtil.ExecIgnoreError("delete from user_token where userId = ?", userId)
}

/**
 * 通过token删除
 * @param token 用户登录票据
 */
func DeleteByToken(token string) {
	DBUtil.ExecIgnoreError("delete from user_token where token = ?", token)
}
