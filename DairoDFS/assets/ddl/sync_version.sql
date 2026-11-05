-- 数据的修改时间(分布式同步用):同一条记录的同一组字段在多台服务器上同时修改时,以最后修改的为准
CREATE TABLE sync_version
(
    tb    VARCHAR(32) NOT NULL, -- 表名
    rowId INT8        NOT NULL, -- 记录ID
    grp   VARCHAR(32) NOT NULL, -- 字段组(同一组的字段总是一起修改,比如文件的父文件夹和文件名)
    ver   INT8        NOT NULL, -- 最后一次修改这组字段的同步日志ID(包含时间和服务器编号,越大越新)
    PRIMARY KEY (tb, rowId, grp)
);
