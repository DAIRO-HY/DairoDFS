#!/usr/bin/env python3
"""
分布式部署(主从、级联、双向同步)的自动化测试

每次修改同步相关的代码之后都要执行一遍。
测试会在临时目录里启动多台全新的DFS(每台有自己的数据目录和存储目录),不会影响任何已有的数据。
双向同步的冲突场景要求两边最终数据一致、文件不丢失。

用法(在项目根目录执行,全部执行完大约15分钟):
    cargo build -p DairoDFS
    python3 document/test/distributed_test.py [DFS程序路径,默认target/debug/DairoDFS]
"""
import hashlib, io, json, os, secrets, shutil, socket, sqlite3, subprocess, sys, tempfile, threading, time
import urllib.error, urllib.parse, urllib.request
from PIL import Image

EXE = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "target/debug/DairoDFS")
ROOT = tempfile.mkdtemp(prefix="dfs-dist-test-")
TABLES = ["user", "user_token", "dfs_file", "dfs_file_delete", "share", "storage_file", "sub_account_grant"]
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


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait(fn, timeout=30, interval=0.3):
    """等待fn返回真,返回最后一次的结果"""
    end = time.time() + timeout
    while True:
        res = fn()
        if res or time.time() > end:
            return res
        time.sleep(interval)


JPG_CACHE = {}


def jpg(seed, size=(800, 600)):
    """同样的参数生成同样的图片"""
    if (seed, size) in JPG_CACHE:
        return JPG_CACHE[(seed, size)]
    import random
    small = Image.frombytes("RGB", (size[0] // 8, size[1] // 8), random.Random(seed).randbytes(size[0] // 8 * size[1] // 8 * 3))
    img = small.resize(size, Image.BILINEAR)
    buf = io.BytesIO()
    img.save(buf, "JPEG", quality=85)
    JPG_CACHE[(seed, size)] = buf.getvalue()
    return buf.getvalue()


class Dfs:
    def __init__(self, name, exe=None):
        self.name = name
        self.exe = exe or EXE
        root = os.path.join(ROOT, name)
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
        self.proc = subprocess.Popen([self.exe, "--web-port", str(self.port), "--data-dir", self.data_dir], stdout=self.log, stderr=self.log)
        for _ in range(200):
            try:
                urllib.request.urlopen(self.url + "/api/install/info", timeout=1)
                return
            except urllib.error.HTTPError:
                return
            except Exception:
                time.sleep(0.1)
        raise RuntimeError(f"{self.name}启动失败")

    def stop(self):
        if self.proc:
            self.proc.terminate()
            self.proc.wait(10)
            self.proc = None

    def request(self, method, path, params=None, body=None, headers=None, raw=False):
        url = self.url + path
        data = body
        h = dict(headers or {})
        if self.token:
            h["Authorization"] = "Bearer " + self.token
        if params is not None:
            query = urllib.parse.urlencode(params, doseq=True)
            if method in ("GET", "DELETE"):
                url += "?" + query
            else:
                data = query.encode()
                h["Content-Type"] = "application/x-www-form-urlencoded"
        req = urllib.request.Request(url, data=data, method=method, headers=h)
        try:
            with urllib.request.urlopen(req, timeout=60) as res:
                body = res.read()
                return res.status, body if raw else body.decode(errors="replace")
        except urllib.error.HTTPError as e:
            return e.code, e.read().decode(errors="replace")

    def ok(self, method, path, params=None, **kw):
        status, text = self.request(method, path, params, **kw)
        assert status == 200, f"{self.name} {method} {path} {params} -> {status} {text[:200]}"
        return json.loads(text) if text and text[0] in "[{" else text

    def install_master(self):
        self.ok("POST", "/api/install/set_storage", {"folders": self.storage_dir})
        self.ok("POST", "/api/install/distributed", {"syncDomains": ""})
        self.ok("POST", "/api/install/create_admin", {"name": "admin", "pwd": "123456"})
        self.login()
        self.profile(openSqlLog=True)

    def install_slave(self, *masters):
        """用安装向导设置为分机,返回是否已经初始化"""
        self.ok("POST", "/api/install/set_storage", {"folders": self.storage_dir})
        status, text = self.request("POST", "/api/install/distributed", {"syncDomains": "\n".join(m.sync_url() for m in masters)})
        # 用户数据同步过来之后用主机的管理员登录
        for _ in range(100):
            try:
                self.login()
                break
            except AssertionError:
                time.sleep(0.3)
        return status, text

    def wait_full_sync(self, timeout=180):
        """等待所有同步主机的全量同步完成(或者出错)"""
        return wait(lambda: all(s["lastTime"] != "无" or s["state"] == 2 for s in self.sync_list()), timeout)

    def login(self, name="admin", pwd="123456", device=None):
        status, text = self.request("POST", "/api/login/do_login", {"name": name, "pwd": pwd, "deviceId": device or f"dist-test-{self.name}"})
        assert status == 200, f"{self.name}登录失败 {status} {text}"
        self.token = json.loads(text)["token"]

    def info(self):
        return self.ok("GET", "/api/profile/info")

    def sync_url(self):
        return f"{self.url}/distributed/{self.info()['token'] if self.token else self.read_config()['DistributedToken']}"

    def read_config(self):
        return json.load(open(os.path.join(self.data_dir, "system.json")))

    def profile(self, **changes):
        """修改系统配置(只修改传入的字段)"""
        info = self.info()
        form = {k: info[k] for k in ["openSqlLog", "isReadOnly", "folders", "syncDomains", "trashTimeout", "deleteStorageTimeout",
                                     "thumbMaxSize", "ignoreSyncError", "dbBackupExpireDay", "operateLogDays", "recycleKeepDays", "nodeNo"] if k in info}
        form.update(changes)
        form = {k: (str(v).lower() if isinstance(v, bool) else v) for k, v in form.items()}
        self.ok("POST", "/api/profile/update", form)

    def sync_list(self):
        return self.ok("GET", "/api/sync/list")

    def upload(self, path, content):
        md5 = hashlib.md5(content).hexdigest()
        status, text = self.request("POST", "/api/file_upload/by_md5", {"md5": md5, "path": path})
        if status == 404:
            self.ok("POST", f"/api/file_upload/by_stream/{md5}", body=content)
            status, text = self.request("POST", "/api/file_upload/by_md5", {"md5": md5, "path": path})
        assert status == 200, f"{self.name}上传{path}失败:{status} {text}"
        return self.file_id(path)

    def list(self, folder):
        return self.ok("GET", "/api/files/list", {"folder": folder})["list"]

    def file_id(self, path):
        folder, name = path.rsplit("/", 1)
        return next(it["id"] for it in self.list(folder or "/") if it["name"] == name)

    def download(self, fid):
        status, body = self.request("GET", f"/api/files/download/{fid}/x", raw=True)
        return body if status == 200 else None

    def db(self):
        con = sqlite3.connect(f"file:{os.path.join(self.data_dir, 'dairo-dfs2.sqlite')}?mode=ro", uri=True, timeout=30)
        return con

    def rows(self, table):
        con = self.db()
        con.row_factory = sqlite3.Row
        res = [dict(it) for it in con.execute(f"select * from {table} order by id")]
        con.close()
        if table == "storage_file":
            for it in res:
                it.pop("path")
        return res

    def wait_handled(self, timeout=120):
        def done():
            con = self.db()
            n = con.execute("select count(*) from dfs_file where isExtra=0 and storageId>0 and state=0 and deleteDate is null").fetchone()[0]
            con.close()
            return n == 0
        return wait(done, timeout, 0.5)

    def log_count(self):
        con = self.db()
        n = con.execute("select count(*), sum(state=0), sum(state=2) from sql_log").fetchone()
        con.close()
        return n


def diff(master, slave, tables=TABLES):
    """对比两台机器的数据,返回不一致的描述,一致时返回空字符串
    user_token:分机本地登录会产生只属于分机的记录,只检查主机的记录分机都有"""
    for table in tables:
        a, b = master.rows(table), slave.rows(table)
        if table == "user_token":
            ids = {it["id"] for it in b}
            missing = [it for it in a if it["id"] not in ids]
            if missing:
                return f"{table}: 分机缺少{len(missing)}条"
            bmap = {it["id"]: it for it in b}
            changed = [it["id"] for it in a if bmap[it["id"]] != it]
            if changed:
                return f"{table}: {len(changed)}条不一致"
            continue
        if a != b:
            amap, bmap = {it["id"]: it for it in a}, {it["id"]: it for it in b}
            only_a = [k for k in amap if k not in bmap]
            only_b = [k for k in bmap if k not in amap]
            changed = [(k, {c: (amap[k][c], bmap[k][c]) for c in amap[k] if amap[k][c] != bmap[k][c]}) for k in amap if k in bmap and amap[k] != bmap[k]]
            return f"{table}: 只在{master.name}{len(only_a)}条{only_a[:3]} 只在{slave.name}{len(only_b)}条{only_b[:3]} 不一致{len(changed)}条{changed[:2]}"
    return ""


def wait_same(master, slave, timeout=40, tables=TABLES):
    res = [None]

    def same():
        res[0] = diff(master, slave, tables)
        return res[0] == ""
    t = time.time()
    wait(same, timeout, 0.3)
    return res[0], time.time() - t


def storage_ok(dfs):
    """存储记录对应的文件都存在且MD5正确"""
    con = dfs.db()
    bad = []
    for sid, path, md5 in con.execute("select id, path, md5 from storage_file"):
        if not os.path.exists(path) or hashlib.md5(open(path, "rb").read()).hexdigest() != md5:
            bad.append(sid)
    con.close()
    return bad


def recycle(dfs):
    """执行一次存储回收,返回回收任务的状态"""
    before = dfs.ok("GET", "/api/advanced/info")["recycleStorageTimerState"]
    time.sleep(1.1)
    dfs.ok("POST", "/api/advanced/recycle_now")
    state = [before]

    def done():
        state[0] = dfs.ok("GET", "/api/advanced/info")["recycleStorageTimerState"]
        return "运行中" not in state[0] and state[0] != before
    wait(done, 60)
    return state[0]


# ---------------------------------------------------------------- 场景
NODES = {}


def node(name):
    it = Dfs(name)
    it.start()
    NODES[name] = it
    return it


def scenario_basic():
    print("\n[1 主机A准备数据,分机B用安装向导加入,全量同步]")
    a = node("A")
    a.install_master()
    a.ok("POST", "/api/files/create_folder", {"folder": "/照片/2024"})
    a.ok("POST", "/api/files/create_folder", {"folder": "/文档"})
    files = {}
    for i in range(6):
        files[f"/照片/2024/p{i}.jpg"] = jpg(i)
    files["/文档/a.txt"] = b"hello dfs" * 1000
    files["/文档/big.bin"] = os.urandom(12 * 1024 * 1024)
    files["/文档/空文件.txt"] = b""
    for path, data in files.items():
        a.upload(path, data)
    a.ok("POST", "/api/user/edit", {"id": 0, "name": "user2", "pwd": "123456", "state": 1})
    sub_id = a.ok("POST", "/api/sub_account/edit", {"id": 0, "name": "kid", "pwd": "abc12345", "state": 1, "paths": ["/照片"], "perms": [1]})
    share_id = a.ok("POST", "/api/files/share", {"ids": a.file_id("/文档/a.txt"), "endDateTime": 0})
    a.ok("DELETE", "/api/files/delete", {"ids": a.file_id("/照片/2024/p5.jpg")})
    check("A后台处理完成(缩略图等)", a.wait_handled())
    a.ok("DELETE", "/api/trash/delete", {"ids": trash_id(a, "p5.jpg")})

    b = node("B")
    status, text = b.install_slave(a)
    check(f"安装向导返回已初始化 {status} {text}", status == 200 and text.strip() == "true")
    check(f"B全量同步完成 {b.sync_list()}", b.wait_full_sync() and b.sync_list()[0]["state"] != 2)
    check("B可以用主机的管理员登录", b.token is not None)
    res, t = wait_same(a, b, 30)
    check(f"A和B数据一致 {res}", res == "")
    check("B的集群ID与A一致", wait(lambda: b.read_config()["ClusterId"] == a.read_config()["ClusterId"], 15))
    bad = storage_ok(b)
    check(f"B的存储文件都存在且MD5正确 {bad}", not bad)
    for path, data in files.items():
        if path.endswith("p5.jpg"):
            continue
        got = b.download(b.file_id(path))
        check(f"B下载{path}内容正确", got == data, f"{got if got is None else len(got)} != {len(data)}")
    status, thumb = b.request("GET", f"/api/files/thumb/{b.file_id('/照片/2024/p0.jpg')}", raw=True)
    check(f"B缩略图可用 {status} {len(thumb)}", status == 200 and thumb[:2] == b"\xff\xd8")
    check("B上没有.full_sync_pending", not os.path.exists(os.path.join(b.data_dir, ".full_sync_pending")))
    b.login("kid", "abc12345")
    check("B可以用子账户登录", True)
    names = [it["name"] for it in b.list("/")]
    check(f"子账户在B上能看到授权的文件夹 {names}", "照片" in names)
    b.login()
    status, text = b.request("GET", f"/api/share/{share_id}/info")
    return a, b


def scenario_realtime(a, b):
    print("\n[2 B设为只读,A上各种操作实时同步到B]")
    b.profile(isReadOnly=True)
    ops = []
    t0 = time.time()
    a.ok("POST", "/api/files/create_folder", {"folder": "/实时"})
    res, t = wait_same(a, b, 15)
    check(f"新建文件夹同步到B 用时{t:.2f}秒 {res}", res == "" and t < 5)
    fid = a.upload("/实时/new.jpg", jpg(100))
    check("A处理新照片", a.wait_handled())
    res, t = wait_same(a, b, 30)
    check(f"上传照片(含缩略图等附属文件)同步到B {res}", res == "")
    check("B下载新照片内容正确", b.download(fid) == jpg(100))
    steps = [
        ("重命名", lambda: a.ok("PUT", "/api/files/rename", {"id": fid, "name": "renamed.jpg"})),
        ("修改拍摄时间", lambda: a.ok("PUT", "/api/files/capture_time", {"id": fid, "date": 946684800000})),
        ("修改文件类型", lambda: a.ok("PUT", "/api/files/content_type", {"id": a.file_id("/文档/a.txt"), "contentType": "text/x-test"})),
        ("复制", lambda: a.ok("POST", "/api/files/copy", {"ids": [fid, a.file_id("/文档/a.txt")], "targetFolder": "/复制", "isOverWrite": "false"})),
        ("移动", lambda: a.ok("POST", "/api/files/move", {"ids": a.file_id("/复制/a.txt"), "targetFolder": "/实时", "isOverWrite": "false"})),
        ("覆盖上传(产生历史版本)", lambda: a.upload("/文档/a.txt", b"version 2")),
        ("删除到回收站", lambda: a.ok("DELETE", "/api/files/delete", {"ids": a.file_id("/复制")})),
        ("从回收站恢复", lambda: a.ok("PUT", "/api/trash/recover", {"ids": trash_id(a, "复制")})),
        ("彻底删除", lambda: (a.ok("DELETE", "/api/files/delete", {"ids": a.file_id("/实时/a.txt")}), a.ok("DELETE", "/api/trash/delete", {"ids": trash_id(a, "a.txt")}))),
        ("新建分享", lambda: a.ok("POST", "/api/files/share", {"ids": fid, "endDateTime": 0, "pwd": "1234"})),
        ("删除分享", lambda: a.ok("DELETE", "/api/my_share/delete", {"ids": [it["id"] for it in a.ok("GET", "/api/my_share/list")][:1]})),
        ("修改子账户授权", lambda: a.ok("POST", "/api/sub_account/edit", {"id": kid_id(a), "name": "kid", "state": 1, "paths": ["/照片", "/实时"], "perms": [1, 3]})),
        ("修改昵称", lambda: a.ok("PUT", "/api/mine/nickname", {"nickname": "爸爸"})),
        ("上传头像", lambda: avatar(a)),
        ("修改用户", lambda: a.ok("POST", "/api/user/edit", {"id": user_id(a, "user2"), "name": "user2", "email": "u2@test.local", "state": 1})),
    ]
    for name, fn in steps:
        fn()
        res, t = wait_same(a, b, 15)
        check(f"{name} 同步到B 用时{t:.2f}秒 {res}", res == "")
    check("A处理完成", a.wait_handled())
    res, t = wait_same(a, b, 30)
    check(f"最终A和B一致 {res}", res == "")
    pa = a.ok("GET", "/api/files/property", {"ids": fid})
    pb = b.ok("GET", "/api/files/property", {"ids": fid})
    check(f"B上的文件属性(含修改后的拍摄时间)与A一致", pa == pb and "946684800000" in json.dumps(pa), f"{pa} {pb}")
    n = b.log_count()
    check(f"B的同步日志没有未执行和失败的 {n}", (n[1] or 0) == 0 and (n[2] or 0) == 0)

    print("\n[3 只读分机B]")
    for name, method, path, params in [
        ("新建文件夹", "POST", "/api/files/create_folder", {"folder": "/b写"}),
        ("重命名", "PUT", "/api/files/rename", {"id": b.file_id("/实时/renamed.jpg"), "name": "x.jpg"}),
        ("删除", "DELETE", "/api/files/delete", {"ids": b.file_id("/实时/renamed.jpg")}),
        ("新增用户", "POST", "/api/user/edit", {"id": 0, "name": "bad", "pwd": "123456", "state": 1}),
    ]:
        status, text = b.request(method, path, params)
        check(f"只读B拒绝{name} {status} {text[:40]}", status != 200 and "只读" in text)
    status, text = b.request("POST", "/api/file_upload/by_md5", {"md5": hashlib.md5(b"x").hexdigest(), "path": "/b.txt"})
    check(f"只读B拒绝上传 {status}", status != 200)
    b.login()
    check("只读B仍可以登录", True)
    res, _ = wait_same(a, b, 5)
    check(f"只读B的拒绝操作没有影响数据 {res}", res == "")


def trash_id(dfs, name):
    return next(it["id"] for it in dfs.ok("GET", "/api/trash/list") if it["name"] == name)


def kid_id(dfs):
    return next(it["id"] for it in dfs.ok("GET", "/api/sub_account/list") if it["name"] == "kid")


def user_id(dfs, name):
    return next(it["id"] for it in dfs.ok("GET", "/api/user/list") if it["name"] == name)


def avatar(dfs):
    boundary = "----dfs" + secrets.token_hex(8)
    body = (f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.jpg\"\r\n"
            f"Content-Type: image/jpeg\r\n\r\n").encode() + jpg(9, (200, 200)) + f"\r\n--{boundary}--\r\n".encode()
    return dfs.ok("POST", "/api/mine/avatar", body=body, headers={"Content-Type": f"multipart/form-data; boundary={boundary}"})


def scenario_offline(a, b):
    print("\n[4 分机B停机期间A继续写,B重启后追上]")
    b.stop()
    for i in range(5):
        a.upload(f"/离线/f{i}.jpg", jpg(200 + i))
    a.ok("PUT", "/api/files/rename", {"id": a.file_id("/离线/f0.jpg"), "name": "f0-改.jpg"})
    a.ok("DELETE", "/api/files/delete", {"ids": a.file_id("/离线/f1.jpg")})
    check("A处理完成", a.wait_handled())
    b.start()
    res, t = wait_same(a, b, 60)
    check(f"B重启后追上A 用时{t:.1f}秒 {res}", res == "")

    print("\n[5 主机A重启,B自动重连]")
    a.stop()
    st = wait(lambda: "失败" in b.sync_list()[0]["msg"], 20)
    check(f"A停机时B显示连接失败 {b.sync_list()[0]['msg'][:60]}", bool(st))
    a.start()
    time.sleep(1)
    a.ok("POST", "/api/files/create_folder", {"folder": "/重启后"})
    res, t = wait_same(a, b, 40)
    check(f"A重启后的修改同步到B 用时{t:.1f}秒 {res}", res == "")
    check(f"B状态正常 {b.sync_list()}", b.sync_list()[0]["state"] != 2)

    print("\n[6 同步过程中网络中断:大文件下载到一半A停机]")
    big = os.urandom(150 * 1024 * 1024)
    fid = a.upload("/离线/huge.bin", big)
    # 等B开始下载
    st = wait(lambda: "正在同步文件" in b.sync_list()[0]["msg"], 30, 0.05)
    msg = b.sync_list()[0]["msg"]
    a.stop()
    time.sleep(4)
    temp = [f for f in os.listdir(os.path.join(b.data_dir, "temp")) if f.startswith("sync-")] if os.path.exists(os.path.join(b.data_dir, "temp")) else []
    print(f"    下载中断时 B: {msg}  临时文件: {temp}  状态: {b.sync_list()[0]['msg'][:50]}")
    a.start()
    res, t = wait_same(a, b, 120)
    check(f"A恢复后B续传完成 {res}", res == "")
    check("B上的大文件内容正确", b.download(fid) == big)
    bad = storage_ok(b)
    check(f"B存储文件MD5全部正确 {bad}", not bad)


def scenario_cascade(a, b):
    print("\n[7 级联:C从B同步(A→B→C)]")
    b.profile(openSqlLog=True)
    c = node("C")
    status, text = c.install_slave(b)
    check(f"C安装向导 {status} {text}", status == 200)
    c.wait_full_sync()
    print(f"    C状态: {c.sync_list()}")
    c.profile(isReadOnly=True)
    res, t = wait_same(a, c, 60)
    check(f"C全量同步后与A一致 {res}", res == "")
    a.ok("POST", "/api/files/create_folder", {"folder": "/级联"})
    fid = a.upload("/级联/c.jpg", jpg(300))
    a.ok("PUT", "/api/files/rename", {"id": fid, "name": "c2.jpg"})
    a.wait_handled()
    res, t = wait_same(a, c, 40)
    check(f"A的修改经过B同步到C 用时{t:.1f}秒 {res}", res == "")
    check("C下载内容正确", c.download(fid) == jpg(300))
    check("C的集群ID与B(也就是A)一致", wait(lambda: c.read_config()["ClusterId"] == a.read_config()["ClusterId"], 10))
    return c


def scenario_token(a, b):
    print("\n[8 票据校验]")
    master_token = a.info()["token"]
    status, text = a.request("GET", f"/distributed/wrong/{secrets.token_hex(8)}/get_aop_id")
    check(f"错误的主机票据被拒绝 {status} {text}", status != 200 and "票据无效" in text)
    status, text = a.request("GET", f"/distributed/{master_token}/{master_token}/get_aop_id")
    check(f"本机循环同步被拒绝 {status} {text}", status != 200 and "循环" in text)
    status, text = a.request("GET", f"/distributed/{master_token}/x/get_table_data", {"tbName": "sql_log", "lastId": 0, "aopId": 2 ** 62})
    check(f"不允许同步的表被拒绝 {status} {text}", status != 200 and "不允许" in text)
    status, text = a.request("GET", f"/distributed/{master_token}/x/get_table_data", {"tbName": "user;drop table user", "lastId": 0, "aopId": 2 ** 62})
    check(f"表名注入被拒绝 {status}", status != 200)
    con = a.db()
    md5 = con.execute("select md5 from storage_file order by id desc limit 1").fetchone()[0]
    con.close()
    status, body = a.request("GET", f"/distributed/{master_token}/x/download/{md5}", headers={"Range": "bytes=10-"}, raw=True)
    check(f"下载接口支持断点续传 {status}", status == 206)
    status, _ = a.request("GET", f"/distributed/{master_token}/x/download/{'0' * 32}")
    check(f"下载不存在的文件返回404 {status}", status == 404)

    print("\n[9 A重新生成票据,B报错;B改成新地址后恢复]")
    a.ok("PUT", "/api/profile/make_token")
    a.ok("POST", "/api/files/create_folder", {"folder": "/新票据"})
    st = wait(lambda: b.sync_list()[0]["state"] == 2, 20)
    check(f"B报错 {b.sync_list()[0]['msg'][:80]}", bool(st))
    b.profile(syncDomains=a.sync_url())
    res, t = wait_same(a, b, 60)
    check(f"B改成新地址后自动全量同步,数据一致 用时{t:.1f}秒 {res}", res == "")
    check(f"B状态正常 {b.sync_list()[0]['msg'][:200]}", wait(lambda: b.sync_list()[0]["state"] != 2, 10))
    con = b.db()
    errs = con.execute("select id, substr(sql,1,50), err from sql_log where state=2").fetchall()
    con.close()
    check(f"没有执行失败的日志 {errs}", not errs)
    a.ok("POST", "/api/files/create_folder", {"folder": "/新票据之后"})
    res, t = wait_same(a, b, 20)
    check(f"之后的修改继续实时同步 用时{t:.1f}秒 {res}", res == "")


def scenario_reset(a, b):
    print("\n[10 只读分机重置数据并重新全量同步]")
    b.profile(isReadOnly=False)
    status, text = b.request("POST", "/api/advanced/reset_data")
    check(f"非只读时不允许重置 {status} {text}", status != 200)
    b.profile(isReadOnly=True)
    b.ok("POST", "/api/advanced/reset_data")
    wait(lambda: b.request("POST", "/api/login/do_login", {"name": "admin", "pwd": "123456", "deviceId": "x"})[0] == 200, 30)
    b.login()
    res, t = wait_same(a, b, 120)
    check(f"重置后重新同步与A一致 用时{t:.1f}秒 {res}", res == "")
    check("同步完成后.full_sync_pending被删除", wait(lambda: not os.path.exists(os.path.join(b.data_dir, ".full_sync_pending")), 20))
    bad = storage_ok(b)
    check(f"重置后B存储文件完整 {bad}", not bad)
    open(os.path.join(b.data_dir, ".full_sync_pending"), "w").write("1")
    state = recycle(b)
    check(f"全量同步未完成时不回收存储文件: {state}", "全量同步还没有完成" in state)
    os.remove(os.path.join(b.data_dir, ".full_sync_pending"))


def scenario_recycle(a, b, c):
    print("\n[11 分机的存储回收不会误删]")
    for it in [b, c]:
        before = it.rows("storage_file")
        state = recycle(it)
        after = it.rows("storage_file")
        con = it.db()
        recycled = con.execute("select count(*) from storage_recycle").fetchone()[0]
        con.close()
        check(f"{it.name}回收后存储文件没有减少 {len(before)}->{len(after)} 隔离区{recycled} 状态:{state}", len(before) == len(after) and recycled == 0)
    # 同步出错时分机不回收
    b.profile(syncDomains=f"{a.url}/distributed/wrongtoken")
    wait(lambda: b.sync_list()[0]["state"] == 2, 20)
    state = recycle(b)
    check(f"同步出错时B不回收存储文件: {state}", "暂不回收" in state)
    b.profile(syncDomains=a.sync_url())
    res, _ = wait_same(a, b, 30)
    check(f"恢复同步地址后一致 {res}", res == "")


def scenario_bidirectional():
    print("\n[12 双向同步:D和E互为主机]")
    d = node("D")
    d.install_master()
    e = node("E")
    status, text = e.install_slave(d)
    e.wait_full_sync()
    e.profile(openSqlLog=True)
    d.profile(syncDomains=e.sync_url())
    res, _ = wait_same(d, e, 30)
    check(f"D和E初始一致 {res}", res == "")
    d.ok("POST", "/api/files/create_folder", {"folder": "/D写"})
    e.ok("POST", "/api/files/create_folder", {"folder": "/E写"})
    fd = d.upload("/D写/d.jpg", jpg(400))
    fe = e.upload("/E写/e.jpg", jpg(401))
    d.wait_handled()
    e.wait_handled()
    res, t = wait_same(d, e, 60)
    check(f"双方的写入互相同步 用时{t:.1f}秒 {res}", res == "")
    check("D能下载E上传的文件", d.download(fe) == jpg(401))
    check("E能下载D上传的文件", e.download(fd) == jpg(400))
    for it in [d, e]:
        n = it.log_count()
        check(f"{it.name}同步日志没有失败的 {n}", (n[2] or 0) == 0)
        check(f"{it.name}同步状态正常 {it.sync_list()}", all(s["state"] != 2 for s in it.sync_list()))
    con = d.db()
    dup = con.execute("select count(*) from (select id from sql_log group by id having count(*)>1)").fetchone()[0]
    con.close()
    check("同步日志没有回环重复", dup == 0)

    print("\n[13 双向同步:两边同时写同一张照片(各自生成附属文件)]")
    same = jpg(500, (1600, 1200))
    d.upload("/D写/same.jpg", same)
    d.wait_handled()
    e.wait_handled()
    res, t = wait_same(d, e, 60)
    check(f"同一张照片双方处理后一致 {res}", res == "")
    con = d.db()
    extras = con.execute("select name, count(*) from dfs_file where parentId=(select id from dfs_file where name='same.jpg' and isExtra=0 and isHistory=0) and isExtra=1 group by name").fetchall()
    con.close()
    check(f"附属文件没有重复 {extras}", extras and all(n == 1 for _, n in extras))

    return d, e


def disconnect(d, e):
    d.profile(syncDomains="")
    e.profile(syncDomains="")


def connect(d, e):
    d.profile(syncDomains=e.sync_url())
    e.profile(syncDomains=d.sync_url())


def sync_state(*nodes):
    return " ".join(f"{it.name}:{it.sync_list()[0]['msg'][:100]}" for it in nodes)


def pair(n1, n2):
    """新建两台互为主机的DFS"""
    d = node(n1)
    d.install_master()
    e = node(n2)
    e.install_slave(d)
    e.wait_full_sync()
    e.profile(openSqlLog=True)
    d.profile(syncDomains=e.sync_url())
    return d, e


def no_failed_log(*nodes):
    for it in nodes:
        n = it.log_count()
        check(f"{it.name}: 同步日志没有失败的 {n}", (n[2] or 0) == 0)


def scenario_conflict():
    print("\n[14 双向同步冲突:断开同步后两边上传同名文件,再恢复]")
    d, e = pair("P1", "Q1")
    d.ok("POST", "/api/files/create_folder", {"folder": "/冲突"})
    res, _ = wait_same(d, e, 20)
    check(f"准备冲突测试的文件夹 {res}", res == "")
    disconnect(d, e)
    d.upload("/冲突/同名文件.txt", b"from d")
    time.sleep(0.01)
    e.upload("/冲突/同名文件.txt", b"from e newer")
    d.wait_handled()
    e.wait_handled()
    connect(d, e)
    res, t = wait_same(d, e, 40)
    print(f"    {sync_state(d, e)}")
    check(f"同名文件合并后两边一致 {res}", res == "")
    for it in [d, e]:
        got = it.download(it.file_id("/冲突/同名文件.txt"))
        check(f"{it.name}: 同名文件保留较新的 {got}", got == b"from e newer")
        con = it.db()
        hist = con.execute("select count(*) from dfs_file where name='同名文件.txt' and isHistory=1").fetchone()[0]
        con.close()
        check(f"{it.name}: 旧的同名文件成为历史版本", hist == 1)
    no_failed_log(d, e)

    print("\n[15 双向同步冲突:同名的文件和文件夹,需要人工处理]")
    d, e = pair("P2", "Q2")
    disconnect(d, e)
    d.ok("POST", "/api/files/create_folder", {"folder": "/同名X"})
    e.upload("/同名X", b"i am a file")
    connect(d, e)
    st = wait(lambda: d.sync_list()[0]["state"] == 2 and e.sync_list()[0]["state"] == 2, 20)
    check(f"双方都报冲突错误 {sync_state(d, e)}", bool(st) and "冲突" in d.sync_list()[0]["msg"] and "冲突" in e.sync_list()[0]["msg"])
    state = recycle(d)
    check(f"冲突出错时不回收存储文件: {state}", "暂不回收" in state)
    check("提示在本机改名", "请在本机" in d.sync_list()[0]["msg"])
    # 按提示在报错的机器上改名,然后重新开始同步
    d.ok("PUT", "/api/files/rename", {"id": d.file_id("/同名X"), "name": "同名X-文件夹"})
    e.ok("PUT", "/api/files/rename", {"id": e.file_id("/同名X"), "name": "同名X.txt"})
    d.ok("POST", "/api/sync/by_log")
    e.ok("POST", "/api/sync/by_log")
    res, t = wait_same(d, e, 30)
    print(f"    改名后 {sync_state(d, e)}")
    check(f"两边改名后同步恢复,两边一致 {res}", res == "")
    for it in [d, e]:
        names = sorted(x["name"] for x in it.list("/"))
        check(f"{it.name}: 文件和文件夹都在 {names}", "同名X-文件夹" in names and "同名X.txt" in names)

    print("\n[16 双向同步冲突:断开同步后两边新建同名文件夹,再恢复]")
    d, e = pair("P3", "Q3")
    disconnect(d, e)
    d.ok("POST", "/api/files/create_folder", {"folder": "/同名文件夹/d内"})
    e.ok("POST", "/api/files/create_folder", {"folder": "/同名文件夹/e内"})
    connect(d, e)
    res, t = wait_same(d, e, 30)
    print(f"    {sync_state(d, e)}")
    check(f"同名文件夹合并后两边一致 {res}", res == "")
    for it in [d, e]:
        names = sorted(x["name"] for x in it.list("/同名文件夹"))
        check(f"{it.name}: 两边的子文件夹都在 {names}", names == ["d内", "e内"])
    no_failed_log(d, e)

    print("\n[16.1 双向同步:在线时两边几乎同时新建同名文件夹(两台手机同时备份到同一个月份)]")
    d, e = pair("P4", "Q4")
    t1 = threading.Thread(target=lambda: d.upload("/照片/2026-11/d.jpg", jpg(700)))
    t2 = threading.Thread(target=lambda: e.upload("/照片/2026-11/e.jpg", jpg(701)))
    t1.start(); t2.start(); t1.join(); t2.join()
    d.wait_handled(); e.wait_handled()
    res, t = wait_same(d, e, 30)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    for it in [d, e]:
        try:
            names = sorted(x["name"] for x in it.list("/照片/2026-11"))
        except AssertionError as ex:
            names = str(ex)[-40:]
        check(f"{it.name}: 两张照片都在 {names}", names == ["d.jpg", "e.jpg"])
    no_failed_log(d, e)

    print("\n[16.2 双向同步:断开同步后两边上传同一张照片(内容相同)]")
    d, e = pair("P5", "Q5")
    disconnect(d, e)
    same = jpg(800)
    fd = d.upload("/d的.jpg", same)
    fe = e.upload("/e的.jpg", same)
    d.wait_handled(); e.wait_handled()
    connect(d, e)
    res, t = wait_same(d, e, 30)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    for it in [d, e]:
        bad = storage_ok(it)
        con = it.db()
        orphan = con.execute("select count(*) from dfs_file f where storageId>0 and not exists(select 1 from storage_file s where s.id=f.storageId)").fetchone()[0]
        con.close()
        check(f"{it.name}: 存储文件完整 {bad}, 没有找不到存储记录的文件 {orphan}", not bad and orphan == 0)
        check(f"{it.name}: 两个文件都能下载", it.download(fd) == same and it.download(fe) == same)
    no_failed_log(d, e)
    for it in [d, e]:
        con = it.db()
        recycled = con.execute("select count(*) from storage_recycle").fetchone()[0]
        con.close()
        state = recycle(it)
        con = it.db()
        recycled2 = con.execute("select count(*) from storage_recycle").fetchone()[0]
        con.close()
        check(f"{it.name}: 回收后没有误回收 {recycled}->{recycled2} {state}", recycled2 == recycled)
        check(f"{it.name}: 回收后两个文件都能下载", it.download(fd) == same and it.download(fe) == same)

    print("\n[16.3 双向同步:同一设备先登录D再登录E]")
    d, e = pair("P6", "Q6")
    d.login(device="iphone")
    e.login(device="iphone")
    time.sleep(3)
    status, _ = d.request("GET", "/api/files/list", {"folder": "/"})
    status2, _ = e.request("GET", "/api/files/list", {"folder": "/"})
    check(f"同一设备在两台上的登录都有效 D:{status} E:{status2}", status == 200 and status2 == 200)
    res, t = wait_same(d, e, 10, ["user_token"])
    check(f"user_token两边一致 {res}", res == "")
    d.token = None
    d.login()
    e.login()

    print("\n[16.4 同时修改:断开时两边修改同一条记录]")
    d, e = pair("P7", "Q7")
    names = {}
    for n in ["f1", "f2", "f3", "a", "b"]:
        names[n] = d.upload(f"/{n}.txt", n.encode() * 10)
    d.ok("POST", "/api/files/create_folder", {"folder": "/目标"})
    d.wait_handled()
    res, _ = wait_same(d, e, 20)
    check(f"准备 {res}", res == "")
    disconnect(d, e)
    d.ok("PUT", "/api/files/rename", {"id": names["f1"], "name": "D改的.txt"})
    time.sleep(0.05)
    e.ok("PUT", "/api/files/rename", {"id": names["f1"], "name": "E改的.txt"})
    d.ok("PUT", "/api/files/rename", {"id": names["f2"], "name": "新名字.txt"})
    e.ok("PUT", "/api/files/content_type", {"id": names["f2"], "contentType": "x/y"})
    d.ok("DELETE", "/api/files/delete", {"ids": names["f3"]})
    time.sleep(0.05)
    e.ok("PUT", "/api/files/rename", {"id": names["f3"], "name": "删除时改名.txt"})
    d.ok("PUT", "/api/files/rename", {"id": names["a"], "name": "撞名.txt"})
    time.sleep(0.05)
    e.ok("PUT", "/api/files/rename", {"id": names["b"], "name": "撞名.txt"})
    connect(d, e)
    res, t = wait_same(d, e, 40)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    for it in [d, e]:
        con = it.db()
        row = lambda fid: con.execute("select name, contentType, deleteDate is not null, isHistory from dfs_file where id=?", (fid,)).fetchone()
        check(f"{it.name}: 同一字段以最后修改的为准 {row(names['f1'])}", row(names["f1"])[0] == "E改的.txt")
        check(f"{it.name}: 不同字段的修改都保留 {row(names['f2'])}", row(names["f2"])[:2] == ("新名字.txt", "x/y"))
        check(f"{it.name}: 删除和改名都保留 {row(names['f3'])}", row(names["f3"])[0] == "删除时改名.txt" and row(names["f3"])[2] == 1)
        ra, rb = row(names["a"]), row(names["b"])
        check(f"{it.name}: 改名撞名时较新的为当前版本,另一个为历史版本 a={ra} b={rb}", ra[0] == rb[0] == "撞名.txt" and ra[3] == 1 and rb[3] == 0)
        con.close()
    no_failed_log(d, e)

    print("\n[16.5 双向同步:断开时两边在多层同名文件夹里添加文件]")
    d, e = pair("P8", "Q8")
    disconnect(d, e)
    d.upload("/M/N/O/d.txt", b"d")
    d.upload("/M/N/同名.txt", b"d same")
    e.upload("/M/N/O/e.txt", b"e")
    e.upload("/M/N/同名.txt", b"e same newer")
    d.wait_handled(); e.wait_handled()
    connect(d, e)
    res, t = wait_same(d, e, 40)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    for it in [d, e]:
        try:
            names = sorted(x["name"] for x in it.list("/M/N/O"))
            same = it.download(it.file_id("/M/N/同名.txt"))
        except AssertionError as ex:
            names, same = str(ex)[-60:], None
        check(f"{it.name}: 多层文件夹合并,文件都在 {names}", names == ["d.txt", "e.txt"])
        check(f"{it.name}: 同名文件保留较新的 {same}", same == b"e same newer")
    no_failed_log(d, e)

    print("\n[16.6 双向同步:断开时两边修改子账户授权]")
    d, e = pair("P9", "Q9")
    d.ok("POST", "/api/files/create_folder", {"folder": "/照片"})
    d.ok("POST", "/api/files/create_folder", {"folder": "/文档"})
    d.ok("POST", "/api/files/create_folder", {"folder": "/视频"})
    kid = d.ok("POST", "/api/sub_account/edit", {"id": 0, "name": "kid", "pwd": "abc12345", "state": 1, "paths": ["/照片"], "perms": [1]})
    res, _ = wait_same(d, e, 20)
    check(f"准备 {res}", res == "")
    disconnect(d, e)
    d.ok("POST", "/api/sub_account/edit", {"id": kid, "name": "kid", "state": 1, "paths": ["/照片", "/文档"], "perms": [3, 1]})
    e.ok("POST", "/api/sub_account/edit", {"id": kid, "name": "kid", "state": 1, "paths": ["/照片", "/视频"], "perms": [1, 1]})
    connect(d, e)
    res, t = wait_same(d, e, 30)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    no_failed_log(d, e)

    print("\n[16.7 服务器编号相同时停止同步]")
    d, e = pair("P10", "Q10")
    check(f"安装向导自动选择了不同的编号 D:{d.info()['nodeNo']} E:{e.info()['nodeNo']}", d.info()["nodeNo"] != e.info()["nodeNo"])
    e.profile(nodeNo=d.info()["nodeNo"])
    st = wait(lambda: e.sync_list()[0]["state"] == 2 and d.sync_list()[0]["state"] == 2, 20)
    check(f"双方都报编号重复 {sync_state(d, e)}", bool(st) and "服务器编号" in e.sync_list()[0]["msg"])
    state = recycle(d)
    check(f"编号重复时不回收存储文件: {state}", "暂不回收" in state)
    e.profile(nodeNo=(d.info()["nodeNo"] + 1) % 10)
    d.ok("POST", "/api/sync/by_log")
    d.ok("POST", "/api/files/create_folder", {"folder": "/改编号之后"})
    res, t = wait_same(d, e, 30)
    check(f"改好编号后同步恢复 {res} {sync_state(d, e)}", res == "")
    con = e.db()
    ids = [r[0] for r in con.execute("select id from dfs_file order by id desc limit 3")]
    con.close()

    print("\n[16.8 双向同步D和E,第三台H只从D同步,也能收到E的修改]")
    d, e = pair("P11", "Q11")
    h = node("H11")
    h.install_slave(d)
    h.wait_full_sync()
    h.profile(isReadOnly=True)
    for i in range(5):
        e.upload(f"/E写/{i}.jpg", jpg(900 + i))
        d.upload(f"/D写/{i}.jpg", jpg(950 + i))
    d.wait_handled(); e.wait_handled()
    res, t = wait_same(d, e, 40)
    check(f"D和E一致 {res}", res == "")
    res, t = wait_same(d, h, 40)
    check(f"H与D一致(E的修改经过D转发) {res}", res == "")
    no_failed_log(d, e, h)

    print("\n[16.9 双向同步:改了地址之后自动全量同步,断开期间两边的修改合并]")
    d, e = pair("P12", "Q12")
    f = d.upload("/x.txt", b"x")
    g = d.upload("/y.txt", b"y")
    d.wait_handled()
    wait_same(d, e, 20)
    disconnect(d, e)
    d.ok("PUT", "/api/files/rename", {"id": f, "name": "D改x.txt"})
    e.ok("PUT", "/api/files/content_type", {"id": f, "contentType": "e/x"})
    e.ok("PUT", "/api/files/rename", {"id": g, "name": "E改y.txt"})
    d.ok("POST", "/api/files/create_folder", {"folder": "/D断开时"})
    e.ok("POST", "/api/files/create_folder", {"folder": "/E断开时"})
    e.ok("PUT", "/api/profile/make_token")
    d.ok("PUT", "/api/profile/make_token")
    connect(d, e)
    res, t = wait_same(d, e, 60)
    print(f"    {sync_state(d, e)}")
    check(f"两边一致 {res}", res == "")
    for it in [d, e]:
        con = it.db()
        rf = con.execute("select name, contentType from dfs_file where id=?", (f,)).fetchone()
        rg = con.execute("select name from dfs_file where id=?", (g,)).fetchone()
        con.close()
        check(f"{it.name}: 两边的修改都保留 {rf} {rg}", rf == ("D改x.txt", "e/x") and rg == ("E改y.txt",))
    no_failed_log(d, e)


def scenario_paging():
    print("\n[16 全量同步分页:主机25000条文件记录]")
    f = node("F")
    f.install_master()
    f.ok("POST", "/api/files/create_folder", {"folder": "/大量"})
    fid = f.file_id("/大量")
    con = sqlite3.connect(os.path.join(f.data_dir, "dairo-dfs2.sqlite"), timeout=30)
    user = con.execute("select userId from dfs_file where id=?", (fid,)).fetchone()[0]
    base = int(time.time() * 1000) - 3600 * 1000
    con.executemany("insert into dfs_file(id,userId,parentId,name,ext,size,contentType,storageId,date,state,isExtra,isHistory) values(?,?,?,?,'',0,'',0,?,1,0,0)",
                    [(base - i, user, fid, f"d{i:05d}", 1700000000000) for i in range(25000)])
    con.commit()
    con.close()
    g = node("G")
    status, text = g.install_slave(f)
    g.wait_full_sync()
    print(f"    G: {g.sync_list()}")
    res, t = wait_same(f, g, 60)
    check(f"25000条记录全量同步完整 {res}", res == "")


def scenario_multi_master(d, e):
    print("\n[17 一台只读备份机H同时从双向同步的D和E同步]")
    h = node("H")
    status, text = h.install_slave(d, e)
    h.wait_full_sync()
    h.profile(isReadOnly=True)
    print(f"    H: {[(s['url'], s['state'], s['msg'][:60]) for s in h.sync_list()]}")
    res, t = wait_same(d, h, 60)
    check(f"H全量同步后与D一致 {res}", res == "")
    d.ok("POST", "/api/files/create_folder", {"folder": "/H测试"})
    wait_same(d, e, 20)
    d.ok("POST", "/api/files/create_folder", {"folder": "/H测试/D"})
    e.ok("POST", "/api/files/create_folder", {"folder": "/H测试/E2"})
    e.upload("/H测试/E2/h.jpg", jpg(600))
    d.wait_handled()
    e.wait_handled()
    res, t = wait_same(d, e, 40)
    check(f"D和E一致 {res}", res == "")
    res, t = wait_same(d, h, 40)
    check(f"H与D一致(同一条日志从两台主机都收到,只执行一次) {res}", res == "")
    n = h.log_count()
    check(f"H的同步日志没有失败的 {n}", (n[2] or 0) == 0)
    check(f"H同步状态正常", all(s["state"] != 2 for s in h.sync_list()), str(h.sync_list()))


def main():
    print(f"测试目录:{ROOT}")
    try:
        a, b = scenario_basic()
        scenario_realtime(a, b)
        scenario_offline(a, b)
        c = scenario_cascade(a, b)
        scenario_token(a, b)
        scenario_reset(a, b)
        scenario_recycle(a, b, c)
        d, e = scenario_bidirectional()
        scenario_multi_master(d, e)
        scenario_conflict()
        scenario_paging()
    except Exception as e:
        import traceback
        traceback.print_exc()
        failed.append(f"异常:{e}")
    finally:
        for it in NODES.values():
            it.stop()
    print(f"\n通过{passed}项,失败{len(failed)}项")
    for it in failed:
        print("  ✗", it)
    print(f"测试目录(日志):{ROOT}")


if __name__ == "__main__":
    main()
