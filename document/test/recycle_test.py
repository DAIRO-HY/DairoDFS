#!/usr/bin/env python3
"""
回收站清理、存储文件回收、隔离区的自动化测试

每次修改回收相关的代码之后都要执行一遍,文件一旦误删就找不回来了。
测试会在临时目录里新建数据目录和存储目录,启动一个DFS,不会影响任何已有的数据。
通过直接修改测试库里的时间来模拟"过了30天"。

用法(在项目根目录执行):
    cargo build -p DairoDFS
    python3 document/test/recycle_test.py [DFS程序路径,默认target/debug/DairoDFS]
"""

import hashlib
import json
import os
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

DAY = 24 * 60 * 60 * 1000

# 测试结果
passed = 0
failed = []


def check(name, ok, detail=""):
    global passed
    if ok:
        passed += 1
        print(f"  ✓ {name}")
    else:
        failed.append(name)
        print(f"  ✗ {name} {detail}")


def now_ms():
    return int(time.time() * 1000)


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Dfs:
    """测试用的DFS服务"""

    def __init__(self, exe, root):
        self.exe = exe
        self.data_dir = os.path.join(root, "data")
        self.storage_dir = os.path.join(root, "storage")
        os.makedirs(self.data_dir)
        os.makedirs(self.storage_dir)
        self.port = free_port()
        self.url = f"http://127.0.0.1:{self.port}"
        self.token = None
        self.proc = None
        self.log = open(os.path.join(root, "dfs.log"), "ab")

    def start(self):
        self.proc = subprocess.Popen(
            [self.exe, "--web-port", str(self.port), "--data-dir", self.data_dir],
            stdout=self.log, stderr=self.log)
        for _ in range(100):
            try:
                urllib.request.urlopen(self.url + "/api/install/info", timeout=1)
                return
            except urllib.error.HTTPError:
                #安装完成之后这个接口会返回错误,能返回说明已经启动
                return
            except Exception:
                time.sleep(0.1)
        raise RuntimeError("DFS启动失败")

    def stop(self):
        if self.proc:
            self.proc.terminate()
            self.proc.wait(10)
            self.proc = None

    def request(self, method, path, params=None, body=None):
        """发送请求,返回(状态码, 内容)"""
        url = self.url + path
        data = body
        headers = {}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        if params is not None:
            query = urllib.parse.urlencode(params, doseq=True)
            if method == "GET":
                url += "?" + query
            else:
                data = query.encode()
                headers["Content-Type"] = "application/x-www-form-urlencoded"
        req = urllib.request.Request(url, data=data, method=method, headers=headers)
        try:
            with urllib.request.urlopen(req, timeout=60) as res:
                return res.status, res.read().decode()
        except urllib.error.HTTPError as e:
            return e.code, e.read().decode()

    def install(self):
        self.request("POST", "/api/install/set_storage", {"folders": self.storage_dir})
        self.request("POST", "/api/install/distributed", {"syncDomains": ""})
        self.request("POST", "/api/install/create_admin", {"name": "admin", "pwd": "123456"})
        self.login()

    def login(self):
        status, text = self.request("POST", "/api/login/do_login", {"name": "admin", "pwd": "123456", "deviceId": "recycle-test"})
        assert status == 200, text
        self.token = json.loads(text)["token"]

    def upload(self, path, content):
        """上传文件,返回文件ID"""
        md5 = hashlib.md5(content).hexdigest()
        status, text = self.request("POST", "/api/file_upload/by_md5", {"md5": md5, "path": path})
        if status == 404:
            status, text = self.request("POST", f"/api/file_upload/by_stream/{md5}", body=content)
            assert status == 200, text
            status, text = self.request("POST", "/api/file_upload/by_md5", {"md5": md5, "path": path})
        assert status == 200, f"上传{path}失败:{status} {text}"
        return self.file_id(path)

    def file_id(self, path):
        folder, name = path.rsplit("/", 1)
        status, text = self.request("GET", "/api/files/list", {"folder": folder})
        assert status == 200, text
        return next(it["id"] for it in json.loads(text)["list"] if it["name"] == name)

    def info(self):
        return json.loads(self.request("GET", "/api/advanced/info")[1])

    def run_recycle(self, api="/api/advanced/recycle_timer"):
        """执行回收任务并等待完成,返回回收状态描述"""
        time.sleep(1.1)  # 状态里的时间精确到秒,保证和上次不同
        before = self.info()["recycleStorageTimerState"]
        self.request("POST", api)
        for _ in range(300):
            time.sleep(0.2)
            state = self.info()["recycleStorageTimerState"]
            if "运行中" not in state and state != before:
                return state
        raise RuntimeError("回收任务没有完成")

    def db(self):
        con = sqlite3.connect(os.path.join(self.data_dir, "dairo-dfs2.sqlite"), timeout=30)
        con.isolation_level = None
        return con


def storage_of(con, file_id):
    """文件对应的存储记录(id, path, md5)"""
    return con.execute("select s.id, s.path, s.md5 from dfs_file f join storage_file s on s.id = f.storageId where f.id = ?", (file_id,)).fetchone()


def age_storage(con, storage_id, days=40):
    """把存储记录的创建时间(ID)改成days天以前,返回新的ID"""
    new_id = storage_id - days * DAY
    con.execute("update storage_file set id = ? where id = ?", (new_id, storage_id))
    con.execute("update dfs_file set storageId = ? where storageId = ?", (new_id, storage_id))
    con.execute("update dfs_file_delete set storageId = ? where storageId = ?", (new_id, storage_id))
    return new_id


def add_unused_storage(dfs, con, name, content, days=40):
    """直接添加一个没有被使用的存储文件,返回(id, path)"""
    folder = os.path.join(dfs.storage_dir, "2020-01")
    os.makedirs(folder, exist_ok=True)
    path = os.path.join(folder, name)
    with open(path, "wb") as f:
        f.write(content)
    storage_id = now_ms() - days * DAY + int.from_bytes(os.urandom(2), "big")
    con.execute("insert into storage_file(id, path, md5) values(?, ?, ?)", (storage_id, path, hashlib.md5(content).hexdigest()))
    return storage_id, path


def recycle_row(con, storage_id):
    return con.execute("select path, recyclePath from storage_recycle where id = ?", (storage_id,)).fetchone()


def exists_storage(con, storage_id):
    return con.execute("select count(*) from storage_file where id = ?", (storage_id,)).fetchone()[0] == 1


def main():
    exe = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "target/debug/DairoDFS")
    root = tempfile.mkdtemp(prefix="dfs-recycle-test-")
    dfs = Dfs(exe, root)
    print(f"测试目录:{root}")
    try:
        dfs.start()
        dfs.install()
        con = dfs.db()
        run_tests(dfs, con)
    finally:
        dfs.stop()
    print(f"\n通过{passed}项,失败{len(failed)}项")
    if failed:
        print("失败的项目:" + ",".join(failed))
        print(f"测试目录保留以便检查:{root}")
        sys.exit(1)
    shutil.rmtree(root)


def run_tests(dfs, con):
    print("1. 回收站:只彻底删除超时的,文件夹连同子文件和历史版本一起处理,存储文件不动")
    old_id = dfs.upload("/t1/old.txt", b"old in trash")
    new_id = dfs.upload("/t1/new.txt", b"new in trash")
    dfs.upload("/t1/folder/c.txt", b"c version 1")
    c_id = dfs.upload("/t1/folder/c.txt", b"c version 2")
    folder_id = dfs.file_id("/t1/folder")
    old_storage = storage_of(con, old_id)
    c_history = con.execute("select id, storageId from dfs_file where isHistory = 1 and name = 'c.txt'").fetchone()
    dfs.request("DELETE", "/api/files/delete", {"ids": [old_id, new_id, folder_id]})
    con.execute("update dfs_file set deleteDate = ? where id in (?, ?)", (now_ms() - 31 * DAY, old_id, folder_id))
    dfs.run_recycle()
    check("超时的文件移到彻底删除表", con.execute("select count(*) from dfs_file_delete where id = ?", (old_id,)).fetchone()[0] == 1)
    check("没超时的文件还在回收站", con.execute("select deleteDate is not null from dfs_file where id = ?", (new_id,)).fetchone() == (1,))
    moved = con.execute("select count(*) from dfs_file_delete where id in (?, ?)", (c_id, c_history[0])).fetchone()[0]
    check("文件夹里的文件和历史版本一起移到彻底删除表", moved == 2, f"实际{moved}")
    check("文件夹记录删除(文件夹没有存储文件)", con.execute("select count(*) from dfs_file where id = ?", (folder_id,)).fetchone()[0] == 0)
    check("彻底删除之后存储文件还在", exists_storage(con, old_storage[0]) and os.path.exists(old_storage[1]))

    print("2. 彻底删除表超时之后,存储文件移到隔离区而不是直接删除")
    aged = age_storage(con, old_storage[0])
    con.execute("update dfs_file_delete set deleteDate = ? where id = ?", (now_ms() - 31 * DAY, old_id))
    dfs.run_recycle()
    row = recycle_row(con, aged)
    check("存储记录已删除", not exists_storage(con, aged))
    check("隔离区有记录", row is not None)
    check("原文件已经移走", not os.path.exists(old_storage[1]))
    check("文件在隔离区里", row is not None and row[1] is not None and os.path.exists(row[1]) and "/.dfs-recycle/" in row[1])
    check("隔离区在存储目录下", row is not None and row[1].startswith(dfs.storage_dir))

    print("3. 正在使用的存储文件不会被回收(即使创建时间很早)")
    active_id = dfs.upload("/t3/active.txt", b"active file")
    active = storage_of(con, active_id)
    active_aged = age_storage(con, active[0])
    same_a = dfs.upload("/t3/same-a.txt", b"same content")
    same_b = dfs.upload("/t3/same-b.txt", b"same content")
    same_storage = storage_of(con, same_a)
    same_aged = age_storage(con, same_storage[0])
    dfs.request("DELETE", "/api/files/delete", {"ids": [same_b]})
    con.execute("update dfs_file set deleteDate = ? where id = ?", (now_ms() - 31 * DAY, same_b))
    history_storage = age_storage(con, c_history[1])
    avatar_id, avatar_path = add_unused_storage(dfs, con, "avatar", b"avatar content")
    con.execute("update user set avatar = ? where name = 'admin'", (hashlib.md5(b"avatar content").hexdigest(),))
    pending_id, pending_path = add_unused_storage(dfs, con, "pending-delete", b"pending in dfs_file_delete")
    con.execute("insert into dfs_file_delete(id, userId, parentId, name, ext, size, storageId, date, isExtra, isHistory, deleteDate, state) "
                "values(9000000000002, 1, 1, 'pending', 'txt', 1, ?, ?, 0, 0, ?, 1)", (pending_id, now_ms(), now_ms()))
    dfs.run_recycle()
    check("正在使用的文件", exists_storage(con, active_aged) and os.path.exists(active[1]))
    check("同一个存储被两个文件使用,删除其中一个", exists_storage(con, same_aged) and os.path.exists(same_storage[1]))
    check("彻底删除表里还没超时的文件", exists_storage(con, history_storage))
    check("头像", exists_storage(con, avatar_id) and os.path.exists(avatar_path))
    check("彻底删除表里引用的存储", exists_storage(con, pending_id) and os.path.exists(pending_path))
    check("文件仍然可以下载", dfs.request("GET", f"/api/files/download/{active_id}/active.txt")[1] == "active file")

    print("4. 隔离区里的文件被使用时自动恢复(模拟回收出错)")
    con.execute("insert into dfs_file(id, userId, parentId, name, ext, size, contentType, storageId, date, isExtra, isHistory, state) "
                "select 9000000000001, userId, parentId, 'restored.txt', 'txt', 10, 'text/plain', ?, date, 0, 0, 1 from dfs_file where id = ?",
                (aged, active_id))
    state = dfs.run_recycle()
    check("存储记录恢复", exists_storage(con, aged))
    check("文件回到原来的位置", os.path.exists(old_storage[1]))
    check("隔离区记录删除", recycle_row(con, aged) is None)
    check("页面显示提示", "自动恢复" in state, state)
    check("恢复之后可以下载", dfs.request("GET", "/api/files/download/9000000000001/restored.txt")[1] == "old in trash")

    print("5. 存储文件被回收之后,秒传返回404,客户端重新上传")
    content = b"recycled then upload again"
    md5 = hashlib.md5(content).hexdigest()
    sid, spath = add_unused_storage(dfs, con, "reupload", content)
    dfs.run_recycle()
    check("存储文件已移到隔离区", recycle_row(con, sid) is not None)
    status, _ = dfs.request("POST", "/api/file_upload/by_md5", {"md5": md5, "path": "/t5/reupload.txt"})
    check("秒传返回404", status == 404, f"实际{status}")
    new_file = dfs.upload("/t5/reupload.txt", content)
    check("重新上传之后内容正确", dfs.request("GET", f"/api/files/download/{new_file}/a.txt")[1] == content.decode())

    print("6. 一次要回收的数量异常时暂停,确认之后才继续")
    total = con.execute("select count(*) from storage_file").fetchone()[0]
    many = [add_unused_storage(dfs, con, f"many-{i}", f"many {i}".encode()) for i in range(60)]
    state = dfs.run_recycle()
    info = dfs.info()
    still = sum(1 for sid, path in many if exists_storage(con, sid) and os.path.exists(path))
    check(f"60个(总数{total + 60})超过1%,暂停回收", still == 60, f"实际剩余{still}")
    check("页面显示需要确认", info.get("recycleBlocked") is not None and "暂停" in state, state)
    dfs.run_recycle("/api/advanced/recycle_confirm")
    gone = sum(1 for sid, _ in many if recycle_row(con, sid) is not None)
    check("确认之后移到隔离区", gone == 60, f"实际{gone}")
    check("确认之后不再提示", dfs.info().get("recycleBlocked") is None)

    print("7. 数据可能不完整时不回收")
    sid, spath = add_unused_storage(dfs, con, "guard", b"guard")
    marker = os.path.join(dfs.data_dir, ".full_sync_pending")
    open(marker, "w").close()
    state = dfs.run_recycle()
    check("全量同步没完成时不回收", exists_storage(con, sid) and "全量同步" in state, state)
    os.remove(marker)
    backup = con.execute("select count(*) from dfs_file").fetchone()[0]
    con.execute("create table dfs_file_backup as select * from dfs_file")
    con.execute("delete from dfs_file")
    state = dfs.run_recycle("/api/advanced/recycle_now")
    check("文件记录为空时不回收", exists_storage(con, sid) and "文件记录为空" in state, state)
    con.execute("insert into dfs_file select * from dfs_file_backup")
    con.execute("drop table dfs_file_backup")
    check("测试数据恢复", con.execute("select count(*) from dfs_file").fetchone()[0] == backup)

    print("8. 隔离区超过保留天数才真正删除")
    keep_id, keep_path = add_unused_storage(dfs, con, "keep", b"keep in recycle")
    old_r_id, old_r_path = add_unused_storage(dfs, con, "old-recycle", b"old in recycle")
    dfs.run_recycle("/api/advanced/recycle_now")
    keep_row = recycle_row(con, keep_id)
    old_row = recycle_row(con, old_r_id)
    con.execute("update storage_recycle set date = ? where id = ?", (now_ms() - 31 * DAY, old_r_id))
    dfs.run_recycle()
    check("没超过保留天数的还在隔离区", recycle_row(con, keep_id) is not None and os.path.exists(keep_row[1]))
    check("超过保留天数的已删除", recycle_row(con, old_r_id) is None and not os.path.exists(old_row[1]))

    print("9. 配置的天数是0时按1天处理,新上传的文件不会被回收")
    dfs.stop()
    config_path = os.path.join(dfs.data_dir, "system.json")
    with open(config_path) as f:
        config = json.load(f)
    config["TrashTimeout"] = 0
    config["DeleteStorageTimeout"] = 0
    config["RecycleKeepDays"] = 0
    with open(config_path, "w") as f:
        json.dump(config, f)
    dfs.start()
    young_id, young_path = add_unused_storage(dfs, con, "young", b"young", days=0)
    trash_id = dfs.upload("/t9/trash.txt", b"trash now")
    dfs.request("DELETE", "/api/files/delete", {"ids": [trash_id]})
    dfs.run_recycle()
    check("刚创建的存储文件不回收", exists_storage(con, young_id) and os.path.exists(young_path))
    check("刚删除的文件还在回收站", con.execute("select count(*) from dfs_file where id = ?", (trash_id,)).fetchone()[0] == 1)
    check("刚移入隔离区的文件不删除", recycle_row(con, keep_id) is not None and os.path.exists(keep_row[1]))

    print("10. 高级页面的三个按钮:清空彻底删除记录 -> 回收未使用文件 -> 清空隔离区,不等待天数")
    gone_id = dfs.upload("/t10/gone.txt", b"gone after purge")
    dfs.upload("/t10/a/shared.txt", b"shared content 10")
    shared_id = dfs.upload("/t10/b/shared.txt", b"shared content 10")
    for fid in [gone_id, shared_id]:
        dfs.request("DELETE", "/api/files/delete", {"ids": [fid]})
    trash = json.loads(dfs.request("GET", "/api/trash/list")[1])
    trash_ids = [it["id"] for it in trash if it["name"] in ("gone.txt", "shared.txt")]
    status, text = dfs.request("DELETE", "/api/trash/delete", {"ids": trash_ids})
    check("从垃圾箱彻底删除", status == 200, f"{status} {text}")
    gone_sid = con.execute("select storageId from dfs_file_delete where id = ?", (gone_id,)).fetchone()[0]
    shared_sid = con.execute("select storageId from dfs_file_delete where id = ?", (shared_id,)).fetchone()[0]

    #存储文件的ID是创建时间,改成1天前创建的(回收未使用文件只回收10分钟之前创建的)
    gone_sid = age_storage(con, gone_sid, days=1)
    shared_sid = age_storage(con, shared_sid, days=1)
    gone_path = con.execute("select path from storage_file where id = ?", (gone_sid,)).fetchone()[0]

    dfs.run_recycle("/api/advanced/recycle_now")
    check("清空记录之前:回收未使用文件不回收彻底删除表里的", exists_storage(con, gone_sid))
    dfs.run_recycle("/api/advanced/clear_file_delete")
    check("清空彻底删除记录:表里没有数据了", con.execute("select count(*) from dfs_file_delete").fetchone()[0] == 0)
    check("清空彻底删除记录:存储文件还没动", exists_storage(con, gone_sid) and os.path.exists(gone_path))
    dfs.run_recycle("/api/advanced/recycle_now")
    row = recycle_row(con, gone_sid)
    check("回收未使用文件:移到了隔离区", row is not None and not exists_storage(con, gone_sid) and os.path.exists(row[1]), row)
    check("还有其他文件在用的存储文件没有回收", exists_storage(con, shared_sid))
    other = [it[0] for it in con.execute("select id from storage_recycle where id != ?", (gone_sid,)).fetchall()]
    dfs.run_recycle("/api/advanced/purge_recycle")
    check("清空隔离区:刚移入的也删除了", recycle_row(con, gone_sid) is None and not os.path.exists(row[1]))
    check("清空隔离区:隔离区没有记录了", con.execute("select count(*) from storage_recycle").fetchone()[0] == 0, other)
    status, _ = dfs.request("GET", f"/api/files/download/{dfs.file_id('/t10/a/shared.txt')}/shared.txt")
    check("共用存储的另一个文件还能下载", status == 200, status)


if __name__ == "__main__":
    main()
