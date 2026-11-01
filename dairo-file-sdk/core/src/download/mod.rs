mod worker;

pub(crate) use worker::part_path;

use crate::error::{SdkError, SdkResult};
use crate::index::{Entry, STATE_DOWNLOADING, STATE_ERROR, STATE_FINISH, STATE_PAUSED, STATE_WAITING};
use crate::options::{CacheMode, Options};
use crate::range_set::RangeSet;
use crate::{key, now_millis, Inner};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/// 要读取的位置在当前下载位置之后这个范围内时,不需要跳过去下载,等一会儿就能下载到
const NEAR_DISTANCE: u64 = 1024 * 1024;

/// 下载任务的数据状态,每写入一段数据就会通知
#[derive(Debug, Clone, Default)]
pub(crate) struct TaskData {
    /// 是否已经开始下载
    pub started: bool,

    /// 文件大小,未知时为None
    pub size: Option<u64>,

    /// 已经写入临时文件的区间
    pub ranges: RangeSet,

    /// 服务器上的文件改变之后会清空已经下载的数据,每清空一次加1,正在读取的请求需要中断
    pub generation: u64,

    /// 服务器返回的文件类型
    pub content_type: Option<String>,

    /// 下载结果,成功时为缓存文件路径
    pub result: Option<Result<String, SdkError>>,
}

/// 一个缓存ID对应的下载任务,同一个文件的多个请求共用一个下载任务
pub(crate) struct Task {
    pub id: String,

    /// 下载完成之后的缓存文件路径
    pub path: PathBuf,

    /// 下载中的临时文件路径
    pub part: PathBuf,

    state: Mutex<TaskState>,

    /// 数据状态通知
    data: watch::Sender<TaskData>,

    /// 停止下载通知
    stop: watch::Sender<bool>,

    /// 期望优先下载的位置(播放器拖动进度时使用)
    want_pos: AtomicU64,

    /// 当前正在下载的位置
    current_pos: AtomicU64,

    /// 当前这次请求开始下载的位置
    segment_start: AtomicU64,

    /// 服务器不支持断点续传,只能从头顺序下载,拖动时不需要跳过去下载
    no_range: AtomicBool,

    /// 被停止之后的状态(等待或暂停)
    stop_state: AtomicI64,
}

struct TaskState {
    /// 最新的下载地址和请求头(地址里的票据可能会过期,总是使用最后一次请求的)
    url: String,
    headers: Vec<(String, String)>,

    /// 所有请求中级别最高的缓存方式
    mode: CacheMode,

    /// 所有请求中最高的优先级
    priority: i32,

    /// 正在等待的请求数,为0时停止下载(永久保存的任务除外)
    waiters: usize,

    /// 暂停或者失败的永久保存任务被播放等请求访问时,只下载需要的部分,保持原来的状态
    hold: Option<i64>,
}

impl Task {
    fn new(inner: &Inner, entry: &Entry, url: &str, options: &Options, waiters: usize) -> Arc<Self> {
        let path = inner.root.join(entry.file());
        let part = part_path(&path);
        let data = TaskData {
            size: entry.size,
            ranges: entry.ranges.clone(),
            content_type: entry.content_type.clone(),
            ..Default::default()
        };
        Arc::new(Self {
            id: entry.id.clone(),
            path,
            part,
            state: Mutex::new(TaskState {
                url: url.to_string(),
                headers: options.headers.clone(),
                mode: options.cache,
                priority: options.priority,
                waiters,
                hold: None,
            }),
            data: watch::Sender::new(data),
            stop: watch::Sender::new(false),
            want_pos: AtomicU64::new(0),
            current_pos: AtomicU64::new(0),
            segment_start: AtomicU64::new(0),
            no_range: AtomicBool::new(false),
            stop_state: AtomicI64::new(STATE_WAITING),
        })
    }

    /// 当前的下载地址和请求头
    pub fn request(&self) -> (String, Vec<(String, String)>) {
        let state = self.state.lock().unwrap();
        (state.url.clone(), state.headers.clone())
    }

    pub fn want_pos(&self) -> u64 {
        self.want_pos.load(Ordering::Relaxed)
    }

    /// 开始一次新的请求
    pub fn set_segment_start(&self, pos: u64) {
        self.segment_start.store(pos, Ordering::Relaxed);
        self.current_pos.store(pos, Ordering::Relaxed);
    }

    pub fn set_current_pos(&self, pos: u64) {
        self.current_pos.store(pos, Ordering::Relaxed);
    }

    /// 需要读取指定位置的数据,离当前下载的位置较远时,跳到这个位置下载
    /// 在这次请求的开始位置到当前位置之后不远的范围内时,数据马上就能读到,不需要跳过去
    /// (当前位置更新之后才会通知数据已经写入,所以位置在当前位置之前也可能暂时读不到)
    pub fn request_pos(&self, pos: u64) {
        let start = self.segment_start.load(Ordering::Relaxed);
        let current = self.current_pos.load(Ordering::Relaxed);
        if pos >= start && pos <= current + NEAR_DISTANCE && self.data.borrow().started {
            return;
        }
        self.want_pos.store(pos, Ordering::Relaxed);
    }

    pub fn set_no_range(&self) {
        self.no_range.store(true, Ordering::Relaxed);
    }

    pub fn is_no_range(&self) -> bool {
        self.no_range.load(Ordering::Relaxed)
    }

    pub fn is_stopped(&self) -> bool {
        *self.stop.borrow()
    }

    /// 等待停止通知
    pub async fn stopped(&self) {
        let mut rx = self.stop.subscribe();
        let _ = rx.wait_for(|it| *it).await;
    }

    pub fn subscribe(&self) -> watch::Receiver<TaskData> {
        self.data.subscribe()
    }

    pub fn update(&self, f: impl FnOnce(&mut TaskData)) {
        self.data.send_modify(f);
    }

    pub fn mode(&self) -> CacheMode {
        self.state.lock().unwrap().mode
    }

    /// 保持的状态
    pub fn hold(&self) -> Option<i64> {
        self.state.lock().unwrap().hold
    }

    /// 事件里的缓存方式,保持状态的任务按缓存下载,但实际是永久保存的文件
    pub fn event_mode(&self) -> CacheMode {
        let state = self.state.lock().unwrap();
        if state.hold.is_some() { CacheMode::Save } else { state.mode }
    }

    fn priority(&self) -> i32 {
        self.state.lock().unwrap().priority
    }
}

/// 一个正在等待下载完成的请求,释放时减少下载任务的等待数
pub(crate) struct Waiter {
    inner: Arc<Inner>,
    pub task: Arc<Task>,
}

impl Drop for Waiter {
    fn drop(&mut self) {
        self.inner.downloads.release(&self.task);
    }
}

/// 请求一个文件的结果
pub(crate) enum Acquired {
    /// 已经缓存
    Ready(Entry, PathBuf),

    /// 正在下载
    Waiting(Waiter),
}

/// 正在运行的任务数
#[derive(Default)]
struct Running {
    /// 等待下载的任务
    pending: Vec<Arc<Task>>,

    /// 正在缓存的任务数
    caching: usize,

    /// 正在永久保存的任务数
    saving: usize,
}

/// 下载管理
#[derive(Default)]
pub(crate) struct Downloads {
    /// 缓存ID => 下载任务
    tasks: Mutex<HashMap<String, Arc<Task>>>,

    running: Mutex<Running>,

    /// 缓存ID => 文件写入锁,同一个文件同时只能有一个任务写入
    file_locks: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

impl Downloads {
    /// 请求一个文件,已经缓存时直接返回路径,否则加入下载
    pub fn acquire(&self, inner: &Arc<Inner>, url: &str, options: &Options) -> SdkResult<Acquired> {
        let (parsed, key) = key::resolve(url, options)?;
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get(&key.id).cloned() {
            //同一个文件正在下载,共用下载任务
            let (upgraded, resumed) = {
                let mut state = task.state.lock().unwrap();
                state.url = url.to_string();
                state.headers = options.headers.clone();
                state.priority = state.priority.max(options.priority);
                state.waiters += 1;
                let upgraded = options.cache > state.mode;
                state.mode = state.mode.max(options.cache);

                //保持状态的任务被要求永久保存时,恢复下载
                let resumed = options.cache == CacheMode::Save && state.hold.take().is_some();
                (upgraded, resumed)
            };
            if upgraded && let Some(mut entry) = inner.index.get(&key.id)? {
                if options.cache > entry.mode {
                    entry.mode = options.cache;
                    entry.group = options.group.clone();
                }
                if resumed {
                    entry.state = if task.data.borrow().started { STATE_DOWNLOADING } else { STATE_WAITING };
                    entry.error = None;
                    inner.events.state(&entry.id, entry.mode, entry.state, None);
                }
                inner.index.save(&entry)?;
            }
            return Ok(Acquired::Waiting(Waiter { inner: inner.clone(), task }));
        }

        let now = now_millis();
        let mut entry = match inner.index.get(&key.id)? {
            Some(mut entry) => {
                let path = inner.root.join(entry.file());
                if entry.state == STATE_FINISH {
                    if path.exists() {
                        if options.cache > entry.mode {
                            entry.mode = options.cache;
                            entry.group = options.group.clone();
                            inner.index.save(&entry)?;
                        }
                        inner.access.record(&inner.index, &entry.id);
                        return Ok(Acquired::Ready(entry, path));
                    }

                    //缓存文件被删除了,重新下载
                    entry.ranges.clear();
                    entry.finish_date = None;
                } else if !part_path(&path).exists() {
                    //临时文件不存在时,之前记录的下载进度无效
                    entry.ranges.clear();
                }
                if options.cache > entry.mode {
                    entry.mode = options.cache;
                    entry.group = options.group.clone();
                }
                entry.url = url.to_string();
                entry.headers = options.headers.clone();
                entry.name = options.name.clone().or(entry.name);
                entry.thumb = options.thumb.clone().or(entry.thumb);
                entry.md5 = options.md5.clone().or(entry.md5);
                entry.access_date = now;
                entry
            }
            None => Entry {
                ext: key::ext(&parsed, options.name.as_deref()),
                id: key.id.clone(),
                namespace: key.namespace,
                key: key.key,
                url: url.to_string(),
                group: options.group.clone(),
                mode: options.cache,
                name: options.name.clone(),
                thumb: options.thumb.clone(),
                size: None,
                ranges: Default::default(),
                etag: None,
                last_modified: None,
                content_type: None,
                md5: options.md5.clone(),
                headers: options.headers.clone(),
                state: STATE_WAITING,
                error: None,
                create_date: now,
                finish_date: None,
                access_date: now,
            },
        };
        //暂停或者失败的永久保存任务被播放等请求访问时,按缓存的方式只下载需要的部分,保持原来的状态
        let hold = (entry.mode == CacheMode::Save
            && (entry.state == STATE_PAUSED || entry.state == STATE_ERROR)
            && options.cache != CacheMode::Save)
            .then_some(entry.state);
        if hold.is_none() {
            entry.state = STATE_WAITING;
            entry.error = None;
        }
        inner.index.save(&entry)?;
        if hold.is_none() {
            inner.events.state(&entry.id, entry.mode, STATE_WAITING, None);
        }
        let mode = if hold.is_some() { CacheMode::Cache } else { entry.mode };
        let task = Task::new(inner, &entry, url, &Options { cache: mode, ..options.clone() }, 1);
        task.state.lock().unwrap().hold = hold;
        tasks.insert(key.id, task.clone());
        drop(tasks);
        self.push(inner, task.clone());
        Ok(Acquired::Waiting(Waiter { inner: inner.clone(), task }))
    }

    /// 继续下载上次没有完成的永久保存任务
    pub fn resume_save(&self, inner: &Arc<Inner>, entry: &Entry) {
        let options = Options {
            cache: CacheMode::Save,
            headers: entry.headers.clone(),
            ..Default::default()
        };
        let task = Task::new(inner, entry, &entry.url, &options, 0);
        self.tasks.lock().unwrap().insert(entry.id.clone(), task.clone());
        self.push(inner, task);
    }

    /// 停止下载任务,不管有没有请求在等待
    /// - state 停止之后的状态(等待或暂停)
    /// - return 是否有正在下载或等待下载的任务
    pub fn stop(&self, inner: &Arc<Inner>, id: &str, state: i64) -> bool {
        let Some(task) = self.tasks.lock().unwrap().remove(id) else {
            return false;
        };
        task.stop_state.store(state, Ordering::Relaxed);
        task.stop.send_replace(true);
        let pending = {
            let mut running = self.running.lock().unwrap();
            let len = running.pending.len();
            running.pending.retain(|it| !Arc::ptr_eq(it, &task));
            len != running.pending.len()
        };
        if pending {
            //还没有开始下载,不会执行run(),这里直接修改状态
            let _ = inner.index.set_state(id, state, None);
            inner.events.state(id, task.event_mode(), state, None);
            task.update(|it| it.result = Some(Err(SdkError::Cancelled)));
        }
        true
    }

    /// 正在下载的任务是保持状态的永久保存任务时,恢复成正常的永久保存下载
    /// - return 是否有正在下载或等待下载的任务
    pub fn resume_held(&self, inner: &Arc<Inner>, id: &str) -> bool {
        let Some(task) = self.tasks.lock().unwrap().get(id).cloned() else {
            return false;
        };
        let resumed = {
            let mut state = task.state.lock().unwrap();
            let resumed = state.hold.take().is_some();
            if resumed {
                state.mode = CacheMode::Save;
            }
            resumed
        };
        if resumed {
            let state = if task.data.borrow().started { STATE_DOWNLOADING } else { STATE_WAITING };
            let _ = inner.index.set_state(id, state, None);
            inner.events.state(id, CacheMode::Save, state, None);
        }
        true
    }

    /// 没有在下载时执行(执行期间不会有新的下载开始),返回是否执行了
    pub fn with_inactive(&self, id: &str, f: impl FnOnce() -> SdkResult<()>) -> SdkResult<bool> {
        let tasks = self.tasks.lock().unwrap();
        if tasks.contains_key(id) {
            return Ok(false);
        }
        f()?;
        drop(tasks);
        Ok(true)
    }

    /// 等待正在写入文件的任务结束
    pub async fn wait_idle(&self, id: &str) {
        let lock = self.file_lock(id);
        let _guard = lock.lock().await;
    }

    /// 释放一个请求,没有请求等待时停止下载
    fn release(&self, task: &Arc<Task>) {
        let mut tasks = self.tasks.lock().unwrap();
        let stop = {
            let mut state = task.state.lock().unwrap();
            state.waiters = state.waiters.saturating_sub(1);
            state.waiters == 0 && state.mode != CacheMode::Save
        };
        if !stop {
            return;
        }
        if tasks.get(&task.id).is_some_and(|it| Arc::ptr_eq(it, task)) {
            tasks.remove(&task.id);
        }
        drop(tasks);
        task.stop.send_replace(true);
        self.running.lock().unwrap().pending.retain(|it| !Arc::ptr_eq(it, task));
    }

    /// 加入等待下载
    fn push(&self, inner: &Arc<Inner>, task: Arc<Task>) {
        self.running.lock().unwrap().pending.push(task);
        self.schedule(inner);
    }

    /// 在并发数允许的范围内,按优先级开始下载等待中的任务
    pub fn schedule(&self, inner: &Arc<Inner>) {
        let mut running = self.running.lock().unwrap();
        loop {
            let config = inner.config();
            let can_cache = running.caching < config.max_caching_count;
            let can_save = running.saving < config.max_saving_count;

            //优先级相同时先加入的先下载
            let next = running
                .pending
                .iter()
                .enumerate()
                .filter(|(_, it)| if it.mode() == CacheMode::Save { can_save } else { can_cache })
                .min_by_key(|(i, it)| (-it.priority(), *i))
                .map(|(i, _)| i);
            let Some(i) = next else {
                break;
            };
            let task = running.pending.remove(i);
            let is_save = task.mode() == CacheMode::Save;
            if is_save {
                running.saving += 1;
            } else {
                running.caching += 1;
            }
            let inner = inner.clone();
            inner.rt.clone().spawn(async move {
                run(&inner, &task).await;
                {
                    let mut running = inner.downloads.running.lock().unwrap();
                    if is_save {
                        running.saving -= 1;
                    } else {
                        running.caching -= 1;
                    }
                }
                inner.downloads.schedule(&inner);
            });
        }
    }

    /// 获取文件写入锁
    fn file_lock(&self, id: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self.file_locks.lock().unwrap();
        locks.retain(|_, it| Arc::strong_count(it) > 1);
        locks.entry(id.to_string()).or_default().clone()
    }
}

/// 执行下载任务
async fn run(inner: &Arc<Inner>, task: &Arc<Task>) {
    let lock = inner.downloads.file_lock(&task.id);
    let _guard = lock.lock().await;
    let result = if task.is_stopped() {
        Err(SdkError::Cancelled)
    } else {
        if task.hold().is_none() {
            let _ = inner.index.set_state(&task.id, STATE_DOWNLOADING, None);
            inner.events.state(&task.id, task.event_mode(), STATE_DOWNLOADING, None);
        }
        worker::download(inner, task).await
    };
    let mode = task.event_mode();
    match (&result, task.hold()) {
        (Ok(_), _) => {
            inner.events.state(&task.id, mode, STATE_FINISH, None);
            crate::cleaner::check_size(inner);
        }
        (Err(_), Some(hold)) => {
            //恢复原来的状态,原来的错误信息不变
            let error = inner.index.get(&task.id).ok().flatten().and_then(|it| it.error);
            let _ = inner.index.set_state(&task.id, hold, error.as_deref());
        }
        (Err(SdkError::Cancelled), None) => {
            let state = task.stop_state.load(Ordering::Relaxed);
            let _ = inner.index.set_state(&task.id, state, None);
            inner.events.state(&task.id, mode, state, None);
        }
        (Err(e), None) => {
            let error = e.to_string();
            let _ = inner.index.set_state(&task.id, STATE_ERROR, Some(&error));
            inner.events.state(&task.id, mode, STATE_ERROR, Some(&error));
        }
    }

    //先从任务列表移除再通知,之后的请求会从缓存索引里读取结果
    {
        let mut tasks = inner.downloads.tasks.lock().unwrap();
        if tasks.get(&task.id).is_some_and(|it| Arc::ptr_eq(it, task)) {
            tasks.remove(&task.id);
        }
    }
    task.update(|it| it.result = Some(result));
}
