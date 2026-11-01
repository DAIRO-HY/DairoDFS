use crate::index::Index;
use crate::now_millis;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 攒够这么多条之后再写入数据库
const MAX_PENDING_COUNT: usize = 500;

/// 最长间隔这么久写入一次数据库
pub const MAX_PENDING_TIME: Duration = Duration::from_secs(60);

/// 记录文件的访问时间
/// 列表快速滑动时每秒可能访问上百个文件,每次都写数据库会影响性能,所以先记在内存里,攒一批再写
#[derive(Default)]
pub(crate) struct AccessRecorder {
    pending: Mutex<Option<(HashSet<String>, Instant)>>,
}

impl AccessRecorder {
    pub fn record(&self, index: &Index, id: &str) {
        let flush = {
            let mut pending = self.pending.lock().unwrap();
            let (ids, since) = pending.get_or_insert_with(|| (HashSet::new(), Instant::now()));
            ids.insert(id.to_string());
            ids.len() >= MAX_PENDING_COUNT || since.elapsed() >= MAX_PENDING_TIME
        };
        if flush {
            self.flush(index);
        }
    }

    /// 把记录的访问时间写入数据库
    pub fn flush(&self, index: &Index) {
        let Some((ids, _)) = self.pending.lock().unwrap().take() else {
            return;
        };
        let ids: Vec<String> = ids.into_iter().collect();
        if let Err(e) = index.touch(&ids, now_millis()) {
            eprintln!("-->dairo-file-sdk:更新访问时间失败:{}", e);
        }
    }
}
