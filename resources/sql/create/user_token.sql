-- 文件表
CREATE TABLE user_token
(
    id         INT8 PRIMARY KEY   NOT NULL,-- 主键
    token      VARCHAR(32) UNIQUE NOT NULL, -- 登录Token
    userId     INT8               NOT NULL,-- 用户ID
    clientFlag INT                NOT NULL,-- 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX
    deviceId   VARCHAR(32)        NOT NULL,-- 设备唯一标识
    ip         VARCHAR(20)        NOT NULL,-- 客户端IP地址
    date       DATETIME           NOT NULL, -- 创建日期
    version    INT                NOT NULL -- 客户端版本
);
CREATE INDEX index_userId ON user_token (userId);
