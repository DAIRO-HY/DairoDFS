-- 分享文件表
CREATE TABLE share_file
(
    shareId INT8          NOT NULL,-- 分享ID
    name    VARCHAR(1000) NOT NULL -- 要分享的文件名或文件夹名
);
CREATE INDEX index_shareId ON share_file (shareId);
