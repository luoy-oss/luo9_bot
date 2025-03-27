//! 时间工具

use chrono::{DateTime, Local, Utc};

/// 将时间戳转换为本地时间字符串
pub fn timestamp_to_string(timestamp: i64) -> String {
    // 使用 DateTime::from_timestamp 替代过时的 from_timestamp_opt
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    let local_time: DateTime<Local> = DateTime::from(datetime);
    
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 获取当前时间的格式化字符串
pub fn current_time_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 获取当前日期的格式化字符串
pub fn current_date_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d").to_string()
}