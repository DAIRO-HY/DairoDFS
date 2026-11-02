-- 子账户授权表
CREATE TABLE sub_account_grant
(
    id        INT8 PRIMARY KEY NOT NULL, -- 主键
    subUserId INT8             NOT NULL, -- 子账户的用户ID
    folderId  INT8             NOT NULL, -- 授权的文件夹ID(主账户的文件夹)
    perms     INTEGER          NOT NULL, -- 权限 1:浏览 2:打开/下载 4:上传 8:修改 16:删除 32:彻底删除 64:分享
    date      INT8             NOT NULL  -- 创建日期
);
CREATE UNIQUE INDEX idx_sub_account_grant ON sub_account_grant (subUserId, folderId);
