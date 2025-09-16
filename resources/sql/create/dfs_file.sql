-- 文件表
CREATE TABLE dfs_file
(
    id          INT8 PRIMARY KEY            NOT NULL,           -- 主键
    userId      INT8                        NOT NULL,-- 所属用户ID
    parentId    INT8                        NOT NULL DEFAULT 0, -- 父目录ID,当isExtra=1时，则标识所属文件的id
    name        VARCHAR(256) COLLATE NOCASE NOT NULL,           -- 名称(比较时忽略大小写)
    ext         VARCHAR(16)                 NOT NULL,           -- 文件扩展名（方便查询指定扩展名的文件，比如电影，图片等）
    size        INT8                        NOT NULL,           -- 大小
    contentType VARCHAR(32)                 NULL,               -- 文件类型(文件专用)
    storageId   INT8                        NOT NULL DEFAULT 0, -- 本地文件存储id(文件专用)
    date        INT8                        NOT NULL,           -- 创建日期
    property    TEXT                        NULL,               -- 文件属性，比如图片尺寸，视频分辨率等信息，JSON字符串
    isExtra     INT1                        NOT NULL DEFAULT 0, -- 是否附属文件，比如视频的标清文件，高清文件，PSD图片的预览图片，cr3的预览图片等
    isHistory   INT1                        NOT NULL DEFAULT 0, -- 是否历史版本(文件专用),1:历史版本 0:当前版本
    deleteDate  INT8                        NULL,               -- 删除日期
    state       INT1                        NOT NULL DEFAULT 0, -- 文件处理状态，0：待处理 1：处理完成 2：处理出错，比如视频文件，需要转码；图片需要获取尺寸等信息
    stateMsg    TEXT                        NULL                -- 文件处理出错信息
);
-- sqlite创建索引，在使用链接查询时，可能会拖慢性能，实测300万条数据查询几乎都是几十微妙就能完成，所以没必要创建索引
-- CREATE INDEX idx_userId ON dfs_file (userId);
-- CREATE INDEX idx_isExtra ON dfs_file (isExtra);
-- CREATE INDEX idx_ext ON dfs_file (ext);
-- CREATE INDEX idx_dfs_file_storageId ON dfs_file (storageId);

-- 非历史文件且未删除时，同一文件夹下文件名不允许重复
CREATE UNIQUE INDEX idx_name
    ON dfs_file (parentId, name) WHERE parentId != 0 and isHistory = 0 and deleteDate is null;
