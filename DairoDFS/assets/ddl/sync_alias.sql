-- 合并记录(分布式同步用):多台服务器各自创建的同名文件夹、同样内容的存储文件合并之后,被合并掉的ID对应到保留下来的ID
-- 之后同步过来的数据里引用了被合并掉的ID时(比如在被合并掉的文件夹里添加文件),换成保留下来的ID
CREATE TABLE sync_alias
(
    id       INT8 PRIMARY KEY NOT NULL, -- 被合并掉的ID
    targetId INT8             NOT NULL, -- 保留下来的ID
    tb       VARCHAR(32)      NOT NULL, -- 表名
    date     INT8             NOT NULL  -- 合并时间
);
