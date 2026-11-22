#!/usr/bin/env python3
"""
上传文件处理设置(缩略图、流畅版视频、兼容版视频、视频编码)的自动化测试

测试会在临时目录里新建数据目录和存储目录,启动DFS,不会影响任何已有的数据。
测试视频用本机的ffmpeg生成,DFS也使用PATH里的ffmpeg/ffprobe。

用法(在项目根目录执行):
    cargo build -p DairoDFS
    python3 document/test/handle_options_test.py [新程序,默认target/debug/DairoDFS] [老程序(数据库版本6),不传时跳过升级测试]
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from recycle_test import Dfs  # noqa: E402

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


# ------------------------------ 测试文件 ------------------------------

class Media:
    """用ffmpeg生成测试文件,每个文件加上不同的标记,内容(MD5)不会相同"""

    def __init__(self, folder):
        self.folder = folder
        self.count = 0

    def make(self, ext, size, video, audio, extra=()):
        self.count += 1
        path = os.path.join(self.folder, f"{self.count}.{ext}")
        args = ["ffmpeg", "-v", "error", "-y",
                "-f", "lavfi", "-i", f"testsrc=size={size}:rate=25:duration=1",
                "-f", "lavfi", "-i", f"sine=frequency={300 + self.count * 10}:duration=1"]
        args += list(extra)
        args += ["-c:v", *video, "-c:a", *audio, "-metadata", f"title=t{self.count}", path]
        subprocess.run(args, check=True)
        with open(path, "rb") as f:
            return f.read()

    def avi(self, size="320x240"):
        """手机播放不了:MPEG-4 Part2 + MP3"""
        return self.make("avi", size, ["mpeg4"], ["libmp3lame"])

    def wmv(self, size="1920x1080"):
        """手机播放不了:WMV + WMA"""
        return self.make("wmv", size, ["wmv2"], ["wmav2"])

    def mkv(self, audio="aac", subtitle=False):
        """H.264的mkv:只需要换封装(AAC/MP3)或者转换音频(AC3)"""
        srt = os.path.join(self.folder, "sub.srt")
        with open(srt, "w") as f:
            f.write("1\n00:00:00,000 --> 00:00:01,000\n字幕\n")
        extra = ["-i", srt, "-map", "0:v", "-map", "1:a", "-map", "2:s", "-c:s", "srt"] if subtitle else []
        return self.make("mkv", "640x360", ["libx264", "-pix_fmt", "yuv420p"], [audio], extra)

    def mp4(self, size="1920x1080"):
        """手机能直接播放:H.264 + AAC"""
        return self.make("mp4", size, ["libx264", "-pix_fmt", "yuv420p"], ["aac"])

    def jpg(self):
        self.count += 1
        path = os.path.join(self.folder, f"{self.count}.jpg")
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", f"testsrc=size=800x600:rate=1:duration=1",
                        "-frames:v", "1", "-metadata", f"comment=t{self.count}", path], check=True)
        with open(path, "rb") as f:
            return f.read() + f"t{self.count}".encode()


def probe(path):
    """(封装格式, 视频编码, 宽, 高, 音频编码, 有没有字幕)"""
    out = subprocess.run(["ffprobe", "-v", "error", "-print_format", "json", "-show_format", "-show_streams", path],
                         check=True, capture_output=True).stdout
    data = json.loads(out)
    streams = data["streams"]
    video = next(it for it in streams if it["codec_type"] == "video")
    audio = next((it["codec_name"] for it in streams if it["codec_type"] == "audio"), None)
    has_sub = any(it["codec_type"] == "subtitle" for it in streams)
    return data["format"]["format_name"], video["codec_name"], video["width"], video["height"], audio, has_sub


def codec_tag(path):
    """视频流的编码标记,如hvc1"""
    out = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=codec_tag_string",
                          "-of", "csv=p=0", path], check=True, capture_output=True).stdout
    return out.decode().strip()


# ------------------------------ DFS操作 ------------------------------

def wait_handled(con, fid, timeout=300):
    """等待文件处理完成,返回(state, stateMsg)"""
    end = time.time() + timeout
    while time.time() < end:
        row = con.execute("select state, stateMsg from dfs_file where id = ?", (fid,)).fetchone()
        if row and row[0] != 0:
            return row
        time.sleep(0.3)
    return None


def extras(con, fid):
    """文件的附属文件:{名称: (storageId, 存储路径)}"""
    rows = con.execute("select e.name, e.storageId, s.path from dfs_file e join storage_file s on s.id = e.storageId "
                       "where e.parentId = ? and e.isExtra = 1", (fid,)).fetchall()
    return {name: (sid, path) for name, sid, path in rows}


def wait_extra(con, fid, name, timeout=300):
    end = time.time() + timeout
    while time.time() < end:
        if name in extras(con, fid):
            return True
        time.sleep(0.5)
    return False


def prop(con, fid):
    row = con.execute("select property from dfs_file where id = ?", (fid,)).fetchone()
    return json.loads(row[0]) if row and row[0] else None


def user_row(con, name):
    return con.execute("select handleThumb, handleSmoothVideo, handleCompatVideo, videoCodec from user where name = ?",
                       (name,)).fetchone()


class Session:
    """以某个用户登录的会话(和admin共用同一个DFS)"""

    def __init__(self, dfs, name, pwd="123456"):
        self.dfs = dfs
        status, text = dfs.request("POST", "/api/login/do_login", {"name": name, "pwd": pwd, "deviceId": "handle-test"})
        assert status == 200, text
        self.token = json.loads(text)["token"]

    def upload(self, path, content):
        token = self.dfs.token
        self.dfs.token = self.token
        try:
            return self.dfs.upload(path, content)
        finally:
            self.dfs.token = token


def add_user(dfs, name, **options):
    """添加用户,options是处理设置(handleThumb等),返回(状态码, 内容)"""
    params = {"id": 0, "name": name, "pwd": "123456", "state": 1, **options}
    return dfs.request("POST", "/api/user/edit", params)


def edit_user(dfs, name, **options):
    uid = dfs.db().execute("select id from user where name = ?", (name,)).fetchone()[0]
    return dfs.request("POST", "/api/user/edit", {"id": uid, "name": name, "state": 1, **options})


def user_detail(dfs, name):
    uid = dfs.db().execute("select id from user where name = ?", (name,)).fetchone()[0]
    return json.loads(dfs.request("GET", "/api/user/detail", {"id": uid})[1])


def set_compat_backfill(dfs, value):
    info = json.loads(dfs.request("GET", "/api/profile/info")[1])
    params = {k: info[k] for k in ["openSqlLog", "isReadOnly", "folders", "syncDomains", "trashTimeout", "deleteStorageTimeout",
                                   "thumbMaxSize", "ignoreSyncError", "dbBackupExpireDay", "operateLogDays", "recycleKeepDays", "nodeNo"]}
    params["compatBackfill"] = "true" if value else "false"
    for k, v in params.items():
        if isinstance(v, bool):
            params[k] = "true" if v else "false"
    status, text = dfs.request("POST", "/api/profile/update", params)
    assert status == 200, text


# ------------------------------ 测试 ------------------------------

def test_default(dfs, con, media):
    print("1. 默认设置(全部开启,H.265)")
    fid = dfs.upload("/a/photo.jpg", media.jpg())
    check("图片处理成功", wait_handled(con, fid)[0] == 1)
    check("图片有属性", prop(con, fid) is not None)
    check("图片有缩略图", "thumb" in extras(con, fid))

    fid = dfs.upload("/a/small.avi", media.avi())
    state = wait_handled(con, fid)
    check("avi处理成功", state[0] == 1, state)
    ex = extras(con, fid)
    check("avi有兼容版", "compat" in ex, ex.keys())
    if "compat" in ex:
        fmt, vcodec, w, h, acodec, _ = probe(ex["compat"][1])
        check("兼容版是mp4", "mp4" in fmt.split(","), fmt)
        check("兼容版是H.265,分辨率不变", (vcodec, w, h) == ("hevc", 320, 240), (vcodec, w, h))
        check("兼容版音频是AAC", acodec == "aac", acodec)
        check("H.265带hvc1标记(苹果设备才能播放)", codec_tag(ex["compat"][1]) == "hvc1", codec_tag(ex["compat"][1]))
    check("avi不大于1280,兼容版同时作为流畅版", "preview" in ex and ex["preview"][0] == ex.get("compat", (0,))[0], ex)
    check("avi有缩略图和首帧图", "thumb" in ex and "preview-frame" in ex, ex.keys())
    check("avi有属性", (prop(con, fid) or {}).get("width") == 320, prop(con, fid))

    fid = dfs.upload("/a/sub.mkv", media.mkv(subtitle=True))
    check("带字幕的H.264 mkv处理成功", wait_handled(con, fid)[0] == 1, wait_handled(con, fid))
    ex = extras(con, fid)
    if check_has(ex, "compat", "mkv有兼容版"):
        fmt, vcodec, w, h, acodec, has_sub = probe(ex["compat"][1])
        check("mkv只换封装:视频还是H.264", vcodec == "h264" and "mp4" in fmt.split(","), (fmt, vcodec))
        check("换封装后分辨率不变", (w, h) == (640, 360), (w, h))
        check("mkv换封装:音频AAC直接复制", acodec == "aac", acodec)
        check("兼容版去掉了字幕", not has_sub)
    check("mkv的属性里有时长(视频流里没有时长,用文件的时长)", (prop(con, fid) or {}).get("duration", 0) > 0, prop(con, fid))

    fid = dfs.upload("/a/ac3.mkv", media.mkv(audio="ac3"))
    check("AC3音频的mkv处理成功", wait_handled(con, fid)[0] == 1)
    ex = extras(con, fid)
    if check_has(ex, "compat", "AC3音频的mkv有兼容版"):
        _, vcodec, _, _, acodec, _ = probe(ex["compat"][1])
        check("视频直接复制,音频转成AAC", (vcodec, acodec) == ("h264", "aac"), (vcodec, acodec))

    fid = dfs.upload("/a/big.mp4", media.mp4())
    check("1080p mp4处理成功", wait_handled(con, fid)[0] == 1)
    ex = extras(con, fid)
    check("能直接播放的mp4没有兼容版", "compat" not in ex, ex.keys())
    if check_has(ex, "preview", "1080p mp4有流畅版"):
        _, vcodec, w, h, _, _ = probe(ex["preview"][1])
        check("流畅版是1280x720的H.265", (vcodec, w, h) == ("hevc", 1280, 720), (vcodec, w, h))

    fid = dfs.upload("/a/big.wmv", media.wmv())
    check("1080p wmv处理成功(以前不认识的格式)", wait_handled(con, fid)[0] == 1, wait_handled(con, fid))
    ex = extras(con, fid)
    if check_has(ex, "compat", "wmv有兼容版") and check_has(ex, "preview", "wmv有流畅版"):
        _, vcodec, w, h, _, _ = probe(ex["compat"][1])
        check("wmv兼容版分辨率不变", (vcodec, w, h) == ("hevc", 1920, 1080), (vcodec, w, h))
        _, vcodec, w, h, _, _ = probe(ex["preview"][1])
        check("wmv流畅版是1280", (w, h) == (1280, 720), (w, h))
        check("wmv兼容版和流畅版是两个文件", ex["compat"][0] != ex["preview"][0])
    check("wmv有属性", (prop(con, fid) or {}).get("width") == 1920, prop(con, fid))

    fid = dfs.upload("/a/odd.avi", media.make("avi", "321x241", ["mjpeg"], ["libmp3lame"]))
    check("宽高是奇数的视频处理成功", wait_handled(con, fid)[0] == 1, wait_handled(con, fid))
    ex = extras(con, fid)
    if check_has(ex, "compat", "奇数宽高的视频有兼容版"):
        _, _, w, h, _, _ = probe(ex["compat"][1])
        check("兼容版宽高是偶数", (w, h) == (320, 240), (w, h))

    fid = dfs.upload("/a/code.ts", b"export const a = 1;\n")
    check("TypeScript文件(.ts)不当作视频,处理成功", wait_handled(con, fid)[0] == 1, wait_handled(con, fid))

    album = {it[2] for it in json.loads(dfs.request("GET", "/api/files/album")[1])}
    check("相册里有新认识的视频格式(wmv)和原来的格式(avi、mkv、jpg)", {"wmv", "avi", "mkv", "mp4", "jpg"} <= album, album)
    check("相册里没有.ts文件", "ts" not in album, album)


def test_share(dfs, con):
    print("1.1 分享的视频:清晰度选择用的接口")
    share_id = json.loads(dfs.request("POST", "/api/files/share", {"ids": dfs.file_id("/a"), "endDateTime": 0})[1])
    url = json.loads(dfs.request("GET", "/api/my_share/detail", {"id": share_id})[1])["url"]
    eid = url.split("/")[2]
    token = dfs.token
    dfs.token = None
    try:
        status, text = dfs.request("GET", f"/api/share/{eid}/video_info/a/big.wmv")
        info = json.loads(text) if status == 200 else {}
        check("不登录也能获取分享视频的信息", status == 200 and info.get("name") == "big.wmv", f"{status} {text[:100]}")
        check("返回流畅版和兼容版", sorted(it["name"] for it in info.get("extraList", [])) == ["compat", "preview"], info)
        for name in ["compat", "preview"]:
            status, body = raw_get(dfs, f"/api/share/{eid}/extra/{name}/a/big.wmv")
            check(f"能播放分享视频的{name}", status == 200 and body[4:8] == b"ftyp", status)
        status, _ = raw_get(dfs, f"/api/share/{eid}/extra/preview-frame/a/big.wmv")
        check("其他附属文件不能通过这个接口访问", status == 404, status)
        status, _ = raw_get(dfs, f"/api/share/{eid}/extra/compat/a/不存在.wmv")
        check("不存在的文件返回404", status == 404, status)
    finally:
        dfs.token = token

    dfs.upload("/b/secret.avi", b"not shared")
    pwd_share = json.loads(dfs.request("POST", "/api/files/share", {"ids": dfs.file_id("/a/small.avi"), "endDateTime": 0, "pwd": "1234"})[1])
    eid = json.loads(dfs.request("GET", "/api/my_share/detail", {"id": pwd_share})[1])["url"].split("/")[2]
    dfs.token = None
    try:
        status, _ = raw_get(dfs, f"/api/share/{eid}/extra/compat/small.avi")
        check("有提取码的分享,没有验证提取码时不能播放", status != 200, status)
        status, _ = raw_get(dfs, f"/api/share/{eid}/video_info/small.avi")
        check("有提取码的分享,没有验证提取码时不能获取信息", status != 200, status)
        status, _ = raw_get(dfs, f"/api/share/{eid}/extra/compat/../b/secret.avi")
        check("不能访问分享以外的文件", status != 200, status)
    finally:
        dfs.token = token


def raw_get(dfs, path):
    req = urllib.request.Request(dfs.url + urllib.parse.quote(path))
    try:
        with urllib.request.urlopen(req, timeout=60) as res:
            return res.status, res.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()


def check_has(ex, name, label):
    ok = name in ex
    check(label, ok, list(ex.keys()))
    return ok


def test_options(dfs, con, media):
    print("2. 账号设置")
    status, text = add_user(dfs, "u264", handleThumb=1, handleSmoothVideo=1, handleCompatVideo=1, videoCodec="h264")
    check("添加H.264账号", status == 200, text)
    check("H.264账号的设置保存正确", user_row(con, "u264") == (1, 1, 1, "h264"), user_row(con, "u264"))
    s = Session(dfs, "u264")
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    ex = extras(con, fid)
    if check_has(ex, "compat", "H.264账号的avi有兼容版"):
        check("兼容版是H.264", probe(ex["compat"][1])[1] == "h264", probe(ex["compat"][1]))
    fid = s.upload("/big.mp4", media.mp4())
    wait_handled(con, fid)
    ex = extras(con, fid)
    if check_has(ex, "preview", "H.264账号的1080p mp4有流畅版"):
        check("流畅版也是H.264", probe(ex["preview"][1])[1] == "h264", probe(ex["preview"][1]))

    status, text = add_user(dfs, "nothumb", handleThumb=0, handleSmoothVideo=1, handleCompatVideo=1, videoCodec="h265")
    check("添加不生成缩略图的账号", status == 200, text)
    s = Session(dfs, "nothumb")
    fid = s.upload("/p.jpg", media.jpg())
    check("图片处理成功", wait_handled(con, fid)[0] == 1)
    check("没有缩略图", "thumb" not in extras(con, fid), extras(con, fid).keys())
    check("属性照样生成", prop(con, fid) is not None)
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    ex = extras(con, fid)
    check("视频有兼容版、流畅版,没有缩略图和首帧图", set(ex) == {"compat", "preview"}, ex.keys())

    status, text = add_user(dfs, "nosmooth", handleThumb=1, handleSmoothVideo=0, handleCompatVideo=1, videoCodec="h265")
    s = Session(dfs, "nosmooth")
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    ex = extras(con, fid)
    check("不生成流畅版:avi只有兼容版,没有preview", "compat" in ex and "preview" not in ex, ex.keys())
    fid = s.upload("/big.mp4", media.mp4())
    wait_handled(con, fid)
    check("不生成流畅版:1080p mp4没有preview", "preview" not in extras(con, fid), extras(con, fid).keys())

    status, text = add_user(dfs, "nocompat", handleThumb=1, handleSmoothVideo=1, handleCompatVideo=0, videoCodec="h265")
    s = Session(dfs, "nocompat")
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    ex = extras(con, fid)
    check("不转换兼容版:320的avi没有compat也没有preview", "compat" not in ex and "preview" not in ex, ex.keys())
    fid = s.upload("/big.wmv", media.wmv())
    wait_handled(con, fid)
    ex = extras(con, fid)
    check("不转换兼容版:1080p wmv只有流畅版", "compat" not in ex and "preview" in ex, ex.keys())

    print("3. 老版本接口(handleFile)")
    status, text = add_user(dfs, "legacy", handleFile=0)
    check("只传handleFile=0添加用户", status == 200, text)
    check("全部关闭,编码默认H.265", user_row(con, "legacy") == (0, 0, 0, "h265"), user_row(con, "legacy"))
    s = Session(dfs, "legacy")
    fid = s.upload("/v.avi", media.avi())
    state = wait_handled(con, fid)
    check("全部关闭的账号:处理成功", state[0] == 1, state)
    check("全部关闭的账号:没有任何附属文件", extras(con, fid) == {}, extras(con, fid).keys())
    check("全部关闭的账号:属性照样生成", (prop(con, fid) or {}).get("width") == 320, prop(con, fid))
    status, text = add_user(dfs, "default")
    check("什么都不传添加用户", status == 200, text)
    check("默认全部开启", user_row(con, "default") == (1, 1, 1, "h265"), user_row(con, "default"))

    edit_user(dfs, "u264", handleFile=0)
    check("老接口修改为不处理:全部关闭,编码保持H.264", user_row(con, "u264") == (0, 0, 0, "h264"), user_row(con, "u264"))
    edit_user(dfs, "u264", handleFile=1)
    check("老接口修改为处理:全部开启", user_row(con, "u264") == (1, 1, 1, "h264"), user_row(con, "u264"))
    edit_user(dfs, "u264")
    check("老页面什么都不传:不修改", user_row(con, "u264") == (1, 1, 1, "h264"), user_row(con, "u264"))
    edit_user(dfs, "u264", videoCodec="h265")
    check("只传一项:其他项不变", user_row(con, "u264") == (1, 1, 1, "h265"), user_row(con, "u264"))
    edit_user(dfs, "u264", handleThumb=0, handleFile=1)
    check("同时传了新项和handleFile:按新项", user_row(con, "u264") == (0, 1, 1, "h265"), user_row(con, "u264"))

    detail = user_detail(dfs, "u264")
    check("用户详情返回设置", (detail.get("handleThumb"), detail.get("handleSmoothVideo"), detail.get("handleCompatVideo"),
                              detail.get("videoCodec")) == (0, 1, 1, "h265"), detail)
    detail = json.loads(dfs.request("GET", "/api/user/detail", {"id": 0})[1])
    check("新增用户的详情默认全部开启", (detail.get("handleThumb"), detail.get("handleCompatVideo"), detail.get("videoCodec")) == (1, 1, "h265"), detail)

    status, _ = edit_user(dfs, "u264", handleThumb=2)
    check("开关传2返回错误", status != 200)
    status, _ = edit_user(dfs, "u264", videoCodec="h266")
    check("不认识的编码返回错误", status != 200)
    check("参数错误时设置不变", user_row(con, "u264") == (0, 1, 1, "h265"), user_row(con, "u264"))

    print("4. 设置关闭时复用已有的附属文件")
    jpg = media.jpg()
    avi = media.avi()
    afid = dfs.upload("/reuse/p.jpg", jpg)
    vfid = dfs.upload("/reuse/v.avi", avi)
    wait_handled(con, afid)
    wait_handled(con, vfid)
    s = Session(dfs, "legacy")
    fid = s.upload("/reuse.jpg", jpg)
    wait_handled(con, fid)
    check("全部关闭的账号复用了缩略图", extras(con, fid).get("thumb", (0,))[0] == extras(con, afid)["thumb"][0])
    fid = s.upload("/reuse.avi", avi)
    wait_handled(con, fid)
    a, b = extras(con, fid), extras(con, vfid)
    check("全部关闭的账号复用了兼容版、流畅版、首帧图",
          all(a.get(k, (0,))[0] == b[k][0] for k in ["compat", "preview", "preview-frame", "thumb"]), (a.keys(), b.keys()))


def test_backfill(dfs, con, media):
    print("5. 补转已有的视频")
    add_user(dfs, "late", handleThumb=1, handleSmoothVideo=1, handleCompatVideo=0, videoCodec="h265")
    s = Session(dfs, "late")
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    check("兼容版关闭时没有兼容版", "compat" not in extras(con, fid))
    edit_user(dfs, "late", handleCompatVideo=1)
    time.sleep(3)
    check("系统设置关闭时不补转", "compat" not in extras(con, fid))
    set_compat_backfill(dfs, True)
    check("开启后补转了兼容版", wait_extra(con, fid, "compat"), extras(con, fid).keys())
    check("补转后兼容版同时作为流畅版", wait_extra(con, fid, "preview") and extras(con, fid)["preview"][0] == extras(con, fid)["compat"][0])
    check("进度文件已经完成", wait_file_text(os.path.join(dfs.data_dir, ".compat_after_id"), str(2 ** 63 - 1)))

    #全部完成之后再上传一个兼容版关闭的视频,打开账号设置,重新开启系统设置时从头补一遍
    edit_user(dfs, "late", handleCompatVideo=0)
    fid = s.upload("/v2.avi", media.avi())
    wait_handled(con, fid)
    edit_user(dfs, "late", handleCompatVideo=1)
    set_compat_backfill(dfs, False)
    set_compat_backfill(dfs, True)
    check("重新开启后从头补转", wait_extra(con, fid, "compat"), extras(con, fid).keys())

    add_user(dfs, "off", handleFile=0)
    s = Session(dfs, "off")
    fid = s.upload("/v.avi", media.avi())
    wait_handled(con, fid)
    set_compat_backfill(dfs, False)
    set_compat_backfill(dfs, True)
    wait_file_text(os.path.join(dfs.data_dir, ".compat_after_id"), str(2 ** 63 - 1))
    check("兼容版关闭的账号不补转", "compat" not in extras(con, fid))
    set_compat_backfill(dfs, False)


def wait_file_text(path, text, timeout=120):
    end = time.time() + timeout
    while time.time() < end:
        try:
            with open(path) as f:
                if f.read().strip() == text:
                    return True
        except FileNotFoundError:
            pass
        time.sleep(0.5)
    return False


def test_reset(dfs, con, media):
    print("6. 重置附属文件后按当前设置重新处理")
    s = Session(dfs, "nothumb")
    fid = s.upload("/reset.avi", media.avi())
    wait_handled(con, fid)
    edit_user(dfs, "nothumb", handleThumb=1, handleCompatVideo=0)
    sql = "delete from dfs_file where isExtra = 1;update dfs_file set state = 0 where storageId > 0 and isExtra = 0"
    status, text = dfs.request("POST", "/api/advanced/exec_sql", {"sql": sql})
    check("执行高级页面的重置附属文件SQL", status == 200, text)
    dfs.request("POST", "/api/advanced/re_handle")
    state = wait_handled(con, fid)
    ex = extras(con, fid)
    check("重置后处理成功", state is not None and state[0] == 1, state)
    check("重置后按新设置:有缩略图,没有兼容版", "thumb" in ex and "compat" not in ex, ex.keys())


def test_upgrade(old_exe, new_exe, media):
    print("7. 从数据库版本6升级")
    root = tempfile.mkdtemp(prefix="dfs-handle-upgrade-")
    try:
        dfs = Dfs(old_exe, root)
        dfs.start()
        dfs.install()
        con = dfs.db()
        add_user(dfs, "music", handleFile=0)
        s = Session(dfs, "music")
        music_avi = s.upload("/v.avi", media.avi())
        music_jpg = s.upload("/p.jpg", media.jpg())
        admin_wmv = dfs.upload("/old/v.wmv", media.wmv(size="640x360"))
        admin_avi = dfs.upload("/old/v.avi", media.avi())
        for fid in [admin_wmv, admin_avi]:
            wait_handled(con, fid)
        check("老版本:不处理的账号的文件state=3",
              con.execute("select state from dfs_file where id = ?", (music_avi,)).fetchone()[0] == 3)
        check("老版本:wmv不认识,没有属性", prop(con, admin_wmv) is None)
        check("老版本:小avi没有兼容版", "compat" not in extras(con, admin_avi))
        con.close()
        dfs.stop()

        dfs.exe = new_exe
        dfs.start()
        dfs.login()
        con = dfs.db()
        check("数据库版本是7", con.execute("pragma user_version").fetchone()[0] == 7)
        check("不处理的账号:全部关闭", user_row(con, "music") == (0, 0, 0, "h265"), user_row(con, "music"))
        check("处理的账号:全部开启", user_row(con, "admin") == (1, 1, 1, "h265"), user_row(con, "admin"))
        check("handleFile列还在(升级前的同步日志要用)",
              con.execute("select count(*) from pragma_table_info('user') where name = 'handleFile'").fetchone()[0] == 1)
        for fid in [music_avi, music_jpg]:
            state = wait_handled(con, fid)
            check(f"原来state=3的文件重新处理成功:{fid}", state is not None and state[0] == 1, state)
        check("原来不处理的视频补上了属性", (prop(con, music_avi) or {}).get("width") == 320, prop(con, music_avi))
        check("原来不处理的图片补上了属性", prop(con, music_jpg) is not None)
        check("原来不处理的文件没有附属文件", extras(con, music_avi) == {} and extras(con, music_jpg) == {})
        check("升级后不会自动补转", "compat" not in extras(con, admin_avi))

        set_compat_backfill(dfs, True)
        check("补转:老avi有了兼容版", wait_extra(con, admin_avi, "compat"), extras(con, admin_avi).keys())
        check("补转:老wmv有了兼容版", wait_extra(con, admin_wmv, "compat"), extras(con, admin_wmv).keys())
        check("补转:老wmv补上了属性", (prop(con, admin_wmv) or {}).get("width") == 640, prop(con, admin_wmv))
        check("补转:老wmv的兼容版作为流畅版", extras(con, admin_wmv).get("preview", (0,))[0] == extras(con, admin_wmv)["compat"][0])
        check("补转:不处理的账号不补", "compat" not in extras(con, music_avi))
        con.close()
        dfs.stop()
    finally:
        shutil.rmtree(root, ignore_errors=True)


def main():
    exe = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "target/debug/DairoDFS")
    old_exe = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else None
    root = tempfile.mkdtemp(prefix="dfs-handle-options-")
    media = Media(tempfile.mkdtemp(prefix="dfs-handle-media-"))
    dfs = Dfs(exe, root)
    try:
        dfs.start()
        dfs.install()
        con = dfs.db()
        test_default(dfs, con, media)
        test_share(dfs, con)
        test_options(dfs, con, media)
        test_backfill(dfs, con, media)
        test_reset(dfs, con, media)
        con.close()
        dfs.stop()
        if old_exe:
            test_upgrade(old_exe, exe, media)
        else:
            print("7. 没有传老程序,跳过升级测试")
    finally:
        dfs.stop()
        shutil.rmtree(root, ignore_errors=True)
        shutil.rmtree(media.folder, ignore_errors=True)
    print(f"\n通过:{passed} 失败:{len(failed)}")
    for it in failed:
        print(f"  ✗ {it}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
