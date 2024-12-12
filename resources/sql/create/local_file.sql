-- 文件表
CREATE TABLE local_file
(
    id   INT8 PRIMARY KEY NOT NULL,
    path VARCHAR(256)     NOT NULL, -- 文件路径
    md5  VARCHAR(32)      NOT NULL  -- 文件MD5
);
CREATE INDEX index_md5 ON local_file (md5);
