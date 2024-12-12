-- 文件表
CREATE TABLE user
(
    id            INT8 PRIMARY KEY NOT NULL,-- 主键
    name          VARCHAR(32) NULL unique, -- 用户名
    pwd           CHAR(32), -- 登陆密码
    email         VARCHAR(64), -- 用户电子邮箱
    urlPath       VARCHAR(10) unique, -- 用户文件访问路径前缀
    apiToken      VARCHAR(20) unique, -- API操作TOKEN
    encryptionKey VARCHAR(256),-- 端对端加密密钥
    state         INTEGER          NOT NULL default 1, -- 用户状态 0:禁用 1:正常 2:删除
    date          DATETIME         NOT NULL -- 创建日期
);
