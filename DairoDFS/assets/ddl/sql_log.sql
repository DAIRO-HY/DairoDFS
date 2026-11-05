-- sql日志
CREATE TABLE sql_log
(
    id     INT8 PRIMARY KEY NOT NULL,           -- 主键
    date   INT8             NOT NULL,           -- 日志时间
    sql    text             NOT NULL,-- sql文
    param  text             NOT NULL,           -- 参数Json
    state  INT              NOT NULL default 0, -- 状态 0：待执行 1：执行完成 2：执行失败
    source VARCHAR(64)      NOT NULL,           -- 日志来源IP
    err    text,                                -- 错误消息
    seq    INT8             NOT NULL default 0  -- 本机的日志顺序号(按添加到本机的先后),分机按这个顺序获取日志
);
CREATE INDEX index_date ON sql_log (date);
CREATE UNIQUE INDEX index_seq ON sql_log (seq);
