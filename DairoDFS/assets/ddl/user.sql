-- 文件表
CREATE TABLE user
(
    id            INT8 PRIMARY KEY NOT NULL,-- 主键
    name          VARCHAR(32)      NULL unique, -- 用户名
    pwd           CHAR(32), -- 登陆密码
    email         VARCHAR(64), -- 用户电子邮箱
    urlPath       VARCHAR(10) unique, -- 用户文件访问路径前缀
    apiToken      VARCHAR(20) unique, -- API操作TOKEN
    encryptionKey VARCHAR(256),-- 端对端加密密钥
    state         INTEGER          NOT NULL default 1, -- 用户状态 0:禁用 1:正常 2:删除
    date          INT8             NOT NULL, -- 创建日期
    parentId      INT8             NOT NULL default 0, -- 所属主账户ID,0表示主账户
    nickname      VARCHAR(32), -- 昵称
    avatar        CHAR(32), -- 头像图片的MD5
    handleFile    INTEGER          NOT NULL default 1, -- 已废弃(版本7改为下面几项),保留这一列是为了升级前产生的同步日志还能执行
    handleThumb       INTEGER      NOT NULL default 1, -- 上传的文件是否生成缩略图、预览图、视频首帧图 0:否 1:是
    handleSmoothVideo INTEGER      NOT NULL default 1, -- 是否生成流畅版视频(比1280大的视频缩小到1280、实况照片的视频) 0:否 1:是
    handleCompatVideo INTEGER      NOT NULL default 1, -- 是否将手机原生播放不了的视频转成兼容版(mp4,分辨率不变) 0:否 1:是
    videoCodec        VARCHAR(8)   NOT NULL default 'h265' -- 转码使用的视频编码 h264 h265
);
