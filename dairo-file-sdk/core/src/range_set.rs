/// 已经下载的数据区间集合,区间为[start, end),按start排序且互不相交
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RangeSet {
    ranges: Vec<(u64, u64)>,
}

impl RangeSet {
    /// 从"0-1024,2048-4096"格式解析
    pub fn parse(text: &str) -> Self {
        let mut set = RangeSet::default();
        for it in text.split(',') {
            if let Some((start, end)) = it.split_once('-')
                && let (Ok(start), Ok(end)) = (start.parse(), end.parse())
            {
                set.insert(start, end);
            }
        }
        set
    }

    /// 添加一个区间,与相邻或重叠的区间合并
    pub fn insert(&mut self, start: u64, end: u64) {
        if start >= end {
            return;
        }
        let (mut start, mut end) = (start, end);
        let mut merged = Vec::with_capacity(self.ranges.len() + 1);
        let mut inserted = false;
        for &(s, e) in &self.ranges {
            if e < start {
                merged.push((s, e));
            } else if end < s {
                if !inserted {
                    merged.push((start, end));
                    inserted = true;
                }
                merged.push((s, e));
            } else {
                start = start.min(s);
                end = end.max(e);
            }
        }
        if !inserted {
            merged.push((start, end));
        }
        self.ranges = merged;
    }

    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// 已经下载的数据总大小
    pub fn len(&self) -> u64 {
        self.ranges.iter().map(|(s, e)| e - s).sum()
    }

    /// 从offset开始连续可读的数据长度
    pub fn available(&self, offset: u64) -> u64 {
        self.ranges
            .iter()
            .find(|(s, e)| *s <= offset && offset < *e)
            .map(|(_, e)| e - offset)
            .unwrap_or(0)
    }

    /// 是否已经下载完成
    pub fn is_complete(&self, size: u64) -> bool {
        size == 0 || self.ranges.first() == Some(&(0, size))
    }

    /// 找到从pos开始的第一个没有下载的区间,pos之后都已经下载时从头开始找
    /// - size 文件大小,未知时传None
    /// - return (开始位置, 结束位置),结束位置为None时表示一直到文件末尾
    pub fn first_missing(&self, pos: u64, size: Option<u64>) -> Option<(u64, Option<u64>)> {
        self.missing_from(pos, size).or_else(|| if pos > 0 { self.missing_from(0, size) } else { None })
    }

    fn missing_from(&self, pos: u64, size: Option<u64>) -> Option<(u64, Option<u64>)> {
        let mut start = pos;
        for &(s, e) in &self.ranges {
            if e <= start {
                continue;
            }
            if s <= start {
                start = e;
                continue;
            }
            return Some((start, Some(s)));
        }
        match size {
            Some(size) if start >= size => None,
            Some(size) => Some((start, Some(size))),
            None => Some((start, None)),
        }
    }
}

impl std::fmt::Display for RangeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text: Vec<String> = self.ranges.iter().map(|(s, e)| format!("{}-{}", s, e)).collect();
        write!(f, "{}", text.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_merge() {
        let mut set = RangeSet::default();
        set.insert(10, 20);
        set.insert(30, 40);
        set.insert(0, 5);
        assert_eq!(set.to_string(), "0-5,10-20,30-40");
        set.insert(5, 10);
        assert_eq!(set.to_string(), "0-20,30-40");
        set.insert(15, 35);
        assert_eq!(set.to_string(), "0-40");
        assert_eq!(RangeSet::parse("0-5,10-20,bad,30-40"), RangeSet::parse("30-40,0-5,10-20"));
        assert_eq!(set.len(), 40);
        assert!(set.is_complete(40));
        assert!(!set.is_complete(41));
    }

    #[test]
    fn missing() {
        let set = RangeSet::parse("0-10,20-30");
        assert_eq!(set.first_missing(0, Some(50)), Some((10, Some(20))));
        assert_eq!(set.first_missing(25, Some(50)), Some((30, Some(50))));
        assert_eq!(set.first_missing(25, None), Some((30, None)));
        assert_eq!(RangeSet::parse("0-10,20-50").first_missing(25, Some(50)), Some((10, Some(20))));
        assert_eq!(RangeSet::parse("0-50").first_missing(0, Some(50)), None);
        assert_eq!(RangeSet::default().first_missing(0, None), Some((0, None)));
        assert_eq!(set.available(5), 5);
        assert_eq!(set.available(10), 0);
    }
}
