#!/usr/bin/env python3
"""
按路径删除文件(DELETE /api/files/delete_by_path)的自动化测试

测试会在临时目录里新建数据目录和存储目录,启动一个DFS,不会影响任何已有的数据。

用法(在项目根目录执行):
    cargo build -p DairoDFS
    python3 document/test/delete_by_path_test.py [DFS程序路径,默认target/debug/DairoDFS]
"""

import json
import os
import shutil
import sys
import tempfile
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from recycle_test import DAY, Dfs, add_unused_storage, recycle_row  # noqa: E402

API = "/api/files/delete_by_path"

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


def delete(dfs, *paths):
    status, text = dfs.request("DELETE", API, {"paths": list(paths)})
    return status, (json.loads(text) if status == 200 else text)


def exists(dfs, path):
    folder, name = path.rsplit("/", 1)
    status, text = dfs.request("GET", "/api/files/list", {"folder": folder})
    return status == 200 and any(it["name"] == name for it in json.loads(text)["list"])


def download(dfs, path):
    fid = dfs.file_id(path)
    req = urllib.request.Request(f"{dfs.url}/api/files/download/{fid}/x", headers={"Authorization": "Bearer " + dfs.token})
    with urllib.request.urlopen(req, timeout=30) as res:
        return res.read()


def rows(con, name):
    """同名文件的所有记录(包括历史版本):(isHistory, 是否已删除)"""
    return con.execute("select isHistory, deleteDate is not null from dfs_file where name = ? order by id", (name,)).fetchall()


def main():
    exe = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "target/debug/DairoDFS")
    root = tempfile.mkdtemp(prefix="dfs-delete-by-path-")
    dfs = Dfs(exe, root)
    try:
        dfs.start()
        dfs.install()
        con = dfs.db()

        print("1. 删除文件,历史版本一起移到垃圾箱")
        dfs.upload("/image/logo/a.jpg", b"old logo")
        dfs.upload("/image/logo/a.jpg", b"new logo")
        check("上传后有当前版本和历史版本", rows(con, "a.jpg") == [(1, 0), (0, 0)], rows(con, "a.jpg"))
        status, res = delete(dfs, "/image/logo/a.jpg")
        check("返回200", status == 200, f"{status} {res}")
        check("返回删除数量1", status == 200 and res == {"deleted": 1, "notFound": []}, res)
        check("列表里没有了", not exists(dfs, "/image/logo/a.jpg"))
        check("当前版本和历史版本都在垃圾箱", rows(con, "a.jpg") == [(1, 1), (0, 1)], rows(con, "a.jpg"))
        trash = [it["name"] for it in json.loads(dfs.request("GET", "/api/trash/list")[1])]
        check("垃圾箱里能看到", "a.jpg" in trash, trash)

        print("2. 路径不存在时不报错,重复调用安全")
        status, res = delete(dfs, "/image/logo/a.jpg")
        check("已删除的路径返回200", status == 200 and res == {"deleted": 0, "notFound": ["/image/logo/a.jpg"]}, f"{status} {res}")
        status, res = delete(dfs, "/没有/这个/文件.jpg")
        check("从来没有过的路径返回200", status == 200 and res["deleted"] == 0, f"{status} {res}")

        print("3. 秒传共用存储:删除一个路径不影响另一个")
        same = b"same content " * 100
        dfs.upload("/image/x/one.jpg", same)
        dfs.upload("/image/y/two.jpg", same)
        sids = con.execute("select storageId from dfs_file where name in ('one.jpg', 'two.jpg')").fetchall()
        check("两个文件共用一个存储文件", len(sids) == 2 and sids[0] == sids[1], sids)

        #存储文件的ID是创建时间,立即回收只回收10分钟之前创建的,改成40天前创建的,让它成为回收的候选
        old_sid = sids[0][0] - 40 * DAY
        con.execute("update storage_file set id = ? where id = ?", (old_sid, sids[0][0]))
        con.execute("update dfs_file set storageId = ? where storageId = ?", (old_sid, sids[0][0]))
        sids = [(old_sid,)]
        status, res = delete(dfs, "/image/x/one.jpg")
        check("删除one.jpg", status == 200 and res["deleted"] == 1, f"{status} {res}")
        check("two.jpg还能下载", download(dfs, "/image/y/two.jpg") == same)

        #从垃圾箱彻底删除,再去掉彻底删除表里的记录(模拟过了保留期),执行存储回收
        trash_id = next(it["id"] for it in json.loads(dfs.request("GET", "/api/trash/list")[1]) if it["name"] == "one.jpg")
        status, text = dfs.request("DELETE", "/api/trash/delete", {"ids": trash_id})
        check("从垃圾箱彻底删除one.jpg", status == 200, f"{status} {text}")
        con.execute("delete from dfs_file_delete")

        #对照:一个没有任何记录引用的存储文件,同一次回收里应该被移到隔离区,说明回收确实执行了
        control_id, _ = add_unused_storage(dfs, con, "control", b"control")
        dfs.run_recycle("/api/advanced/recycle_now")
        check("对照:没有引用的存储文件被回收", recycle_row(con, control_id) is not None)
        sid = sids[0][0]
        storage = con.execute("select path from storage_file where id = ?", (sid,)).fetchone()
        check("存储回收后存储记录还在", storage is not None)
        check("存储文件还在磁盘上", storage is not None and os.path.exists(storage[0]))
        check("回收之后two.jpg还能下载", download(dfs, "/image/y/two.jpg") == same)

        print("4. 只能删除文件,文件夹返回400,一个都不删除")
        dfs.upload("/image/keep/b.jpg", b"keep b")
        status, text = delete(dfs, "/image/keep")
        check("文件夹返回400", status == 400, f"{status} {text}")
        check("文件夹里的文件还在", exists(dfs, "/image/keep/b.jpg"))
        status, text = delete(dfs, "/image/keep/b.jpg", "/image")
        check("同时传文件和文件夹返回400", status == 400, f"{status} {text}")
        check("同一个请求里的文件也没有删除", exists(dfs, "/image/keep/b.jpg"))
        status, text = delete(dfs, "/")
        check("根目录返回400", status == 400, f"{status} {text}")
        check("根目录下的文件还在", exists(dfs, "/image/keep/b.jpg"))

        print("5. 参数错误返回400")
        status, text = dfs.request("DELETE", API)
        check("没有传paths返回400", status == 400, f"{status} {text}")
        status, text = delete(dfs, "image/keep/b.jpg")
        check("路径不以/开头返回400", status == 400, f"{status} {text}")
        check("文件还在", exists(dfs, "/image/keep/b.jpg"))

        print("6. 一次删除多个,有的不存在")
        dfs.upload("/image/m/1.jpg", b"m1")
        dfs.upload("/image/m/2.jpg", b"m2")
        status, res = delete(dfs, "/image/m/1.jpg", "/image/m/2.jpg", "/image/m/3.jpg", "/image/m/1.jpg")
        check("删除2个、1个不存在", status == 200 and res == {"deleted": 2, "notFound": ["/image/m/3.jpg"]}, f"{status} {res}")
        check("两个文件都删除了", not exists(dfs, "/image/m/1.jpg") and not exists(dfs, "/image/m/2.jpg"))

        print("7. 子账户按授权检查删除权限")
        dfs.upload("/image/kid/k.jpg", b"kid file")
        admin_token = dfs.token
        status, text = dfs.request("POST", "/api/sub_account/edit", {"id": 0, "name": "kid", "pwd": "abc12345", "state": 1, "paths": ["/image/kid"], "perms": [1 | 2 | 4]})
        check("新建没有删除权限的子账户", status == 200, f"{status} {text}")
        kid_id = int(text)
        status, text = dfs.request("POST", "/api/login/do_login", {"name": "kid", "pwd": "abc12345", "deviceId": "delete-by-path-test"})
        kid_token = json.loads(text)["token"]
        dfs.token = kid_token
        status, text = delete(dfs, "/image/kid/k.jpg")
        check("没有删除权限时拒绝", status != 200, f"{status} {text}")
        status, text = delete(dfs, "/image/keep/b.jpg")
        check("没有授权的文件夹拒绝", status != 200, f"{status} {text}")
        dfs.token = admin_token
        check("文件还在", exists(dfs, "/image/kid/k.jpg") and exists(dfs, "/image/keep/b.jpg"))
        status, text = dfs.request("POST", "/api/sub_account/edit", {"id": kid_id, "name": "kid", "state": 1, "paths": ["/image/kid"], "perms": [1 | 2 | 4 | 16]})
        check("给子账户加上删除权限", status == 200, f"{status} {text}")
        dfs.token = kid_token
        status, res = delete(dfs, "/image/kid/k.jpg")
        check("有删除权限时可以删除", status == 200 and res["deleted"] == 1, f"{status} {res}")
        dfs.token = admin_token
        check("文件已删除", not exists(dfs, "/image/kid/k.jpg"))

        print("8. 没有登录返回401")
        dfs.token = None
        status, _ = delete(dfs, "/image/keep/b.jpg")
        check("没有票据返回401", status == 401, status)
        dfs.token = admin_token
        check("文件还在", exists(dfs, "/image/keep/b.jpg"))
    finally:
        dfs.stop()
    print()
    print(f"通过{passed}项,失败{len(failed)}项")
    if failed:
        print(f"测试目录(日志):{root}")
        sys.exit(1)
    shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    main()
