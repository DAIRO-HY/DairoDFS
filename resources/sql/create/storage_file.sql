-- 本地存储表
CREATE TABLE storage_file
(
    id   INT8 PRIMARY KEY NOT NULL,
    path VARCHAR(256)     NOT NULL, -- 文件路径
    md5  VARCHAR(32)      NOT NULL  -- 文件MD5
);
CREATE INDEX idx_md5 ON storage_file (md5);
