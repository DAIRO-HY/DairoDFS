-- 分享表
CREATE TABLE share
(
    id         INT8 PRIMARY KEY NOT NULL,
    title      VARCHAR(300)     NULL,     -- 分享标题（文件名）
    userId     INT8             NOT NULL,-- 所属用户ID
    pwd        VARCHAR(32)      NULL,     -- 加密分享
    folder     VARCHAR(512)     NOT NULL, -- 分享的文件所属文件夹
    names      TEXT             NOT NULL, -- 分享的文件夹或文件名,用|分割
    thumb      INT8,-- 缩略图
    folderFlag INT1,-- 是否分享的仅仅是一个文件夹
    fileCount  INT              NOT NULL, -- 文件数
    endDate    INT8             NULL,     -- 结束日期
    date       INT8             NOT NULL  -- 创建日期
);
CREATE INDEX index_userId ON share (userId);
