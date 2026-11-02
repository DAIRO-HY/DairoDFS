-- 操作日志表(只记录在本机,不参与分布式同步)
CREATE TABLE operate_log
(
    id         INT8 PRIMARY KEY NOT NULL, -- 主键
    date       INT8             NOT NULL, -- 操作时间
    userId     INT8             NOT NULL, -- 操作人的用户ID
    userName   VARCHAR(32)      NOT NULL, -- 操作人的用户名(操作时)
    ownerId    INT8             NOT NULL, -- 操作人所属的主账户ID,主账户为自己的ID
    opType     INTEGER          NOT NULL, -- 操作类型
    fileId     INT8             NULL,     -- 操作的文件ID
    path       TEXT             NULL,     -- 操作对象的路径(操作时)
    detail     TEXT             NULL,     -- 其他信息,JSON字符串,例如移动的目标路径
    ip         VARCHAR(64)      NOT NULL, -- 客户端IP地址
    clientFlag INTEGER          NOT NULL  -- 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX
);
CREATE INDEX idx_operate_log_owner ON operate_log (ownerId, date);
CREATE INDEX idx_operate_log_date ON operate_log (date);
