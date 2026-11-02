-- 存储文件隔离区:没有被使用的存储文件不直接删除,先移到隔离区,超过保留天数才真正删除,误删时可以恢复
-- 只记录在本机,不参与分布式同步(每台机器各自回收自己的存储文件)
CREATE TABLE storage_recycle
(
    id          INT8 PRIMARY KEY NOT NULL, -- 存储文件ID(和原来的storage_file.id相同,恢复时使用)
    md5         VARCHAR(32)      NOT NULL, -- 文件MD5
    path        VARCHAR(256)     NOT NULL, -- 原来的存储路径
    recyclePath VARCHAR(256)     NULL,     -- 隔离区里的路径,移入隔离区时原文件已经不存在则为空
    size        INT8             NOT NULL, -- 文件大小
    date        INT8             NOT NULL  -- 移入隔离区的时间
);
CREATE INDEX idx_storage_recycle_date ON storage_recycle (date);
