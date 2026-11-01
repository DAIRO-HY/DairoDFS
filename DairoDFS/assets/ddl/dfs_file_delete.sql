-- 彻底删除的文件表(结构与dfs_file一致),用来记录已经彻底删除的文件,方便后续回收本地存储文件
CREATE TABLE dfs_file_delete
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
    isExtra     INTEGER                     NOT NULL DEFAULT 0, -- 是否附属文件，比如视频的标清文件，高清文件，PSD图片的预览图片，cr3的预览图片等
    isHistory   INTEGER                     NOT NULL DEFAULT 0, -- 是否历史版本(文件专用),1:历史版本 0:当前版本
    deleteDate  INT8                        NULL,               -- 删除日期
    state       INTEGER                     NOT NULL DEFAULT 0, -- 文件处理状态，0：待处理 1：处理完成 2：处理出错，比如视频文件，需要转码；图片需要获取尺寸等信息
    stateMsg    TEXT                        NULL                -- 文件处理出错信息
);
