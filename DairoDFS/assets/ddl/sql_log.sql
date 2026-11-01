-- sql日志
CREATE TABLE sql_log
(
    id     INT8 PRIMARY KEY NOT NULL,           -- 主键
    date   INT8             NOT NULL,           -- 日志时间
    sql    text             NOT NULL,-- sql文
    param  text             NOT NULL,           -- 参数Json
    state  INT              NOT NULL default 0, -- 状态 0：待执行 1：执行完成 2：执行失败
    source VARCHAR(64)      NOT NULL,           -- 日志来源IP
    err    text                                 -- 错误消息
);
CREATE INDEX index_date ON sql_log (date);