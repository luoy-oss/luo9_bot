// src/plugin/task.rs
// luo9_task 总线接收 + 轻量 cron 调度器（6 字段，支持 ? L W #）

use std::collections::HashSet;
use chrono::{Datelike, NaiveDate, Timelike};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::bus;

// ── Cron 字段类型 ──────────────────────────────────────────────

enum FieldSet {
    Any,                                    // * 或 ?
    Values(HashSet<u32>),                   // 具体值集合
    LastDay,                                // L（当月最后一天）
    LastWeekday(u32),                       // 5L（当月最后一个周五）
    NearestWeekday(u32),                    // 15W（离 15 号最近的工作日）
    NthWeekday { weekday: u32, n: u32 },    // 2#3（当月第 3 个周一）
}

impl FieldSet {
    fn matches(&self, value: u32, ctx: &MatchContext) -> bool {
        match self {
            FieldSet::Any => true,
            FieldSet::Values(set) => set.contains(&value),
            FieldSet::LastDay => value == ctx.last_day_of_month,
            FieldSet::LastWeekday(w) => {
                let last = last_weekday_of_month(ctx.year, ctx.month, *w);
                last.map_or(false, |d| d == value)
            }
            FieldSet::NearestWeekday(day) => {
                nearest_weekday(ctx.year, ctx.month, *day).map_or(false, |d| d == value)
            }
            FieldSet::NthWeekday { weekday, n } => {
                nth_weekday(ctx.year, ctx.month, *weekday, *n).map_or(false, |d| d == value)
            }
        }
    }
}

struct MatchContext {
    year: i32,
    month: u32,
    last_day_of_month: u32,
}

// ── Cron 表达式解析 ────────────────────────────────────────────

struct CronExpr {
    seconds: FieldSet,
    minutes: FieldSet,
    hours: FieldSet,
    days: FieldSet,
    months: HashSet<u32>,
    weekdays: FieldSet,
}

impl CronExpr {
    fn parse(expr: &str) -> Option<Self> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        if fields.len() != 6 {
            warn!("[task] cron 表达式需要 6 个字段（秒 分 时 日 月 周），实际 {} 个: {}", fields.len(), expr);
            return None;
        }
        Some(Self {
            seconds: parse_std_field(fields[0], 0, 59)?,
            minutes: parse_std_field(fields[1], 0, 59)?,
            hours: parse_std_field(fields[2], 0, 23)?,
            days: parse_day_field(fields[3])?,
            months: parse_month_field(fields[4])?,
            weekdays: parse_weekday_field(fields[5])?,
        })
    }

    fn matches(&self, sec: u32, min: u32, hour: u32, day: u32, month: u32, weekday: u32, year: i32) -> bool {
        let last_day = last_day_of_month(year, month);
        let ctx = MatchContext { year, month, last_day_of_month: last_day };

        // 日和周的互斥规则：任一为 ? 则跳过该字段
        let day_ok = self.days.matches(day, &ctx);
        let wd = if weekday == 0 { 7 } else { weekday }; // chrono: 0=Sun → 7
        let wd_ok = self.weekdays.matches(wd, &ctx);

        self.seconds.matches(sec, &ctx)
            && self.minutes.matches(min, &ctx)
            && self.hours.matches(hour, &ctx)
            && self.months.contains(&month)
            && day_ok
            && wd_ok
    }
}

// ── 标准字段解析（秒、分、时）────────────────────────────────

fn parse_std_field(field: &str, min: u32, max: u32) -> Option<FieldSet> {
    if field == "*" || field == "?" {
        return Some(FieldSet::Any);
    }
    let mut set = HashSet::new();
    for part in field.split(',') {
        parse_range_part(part, min, max, &mut set)?;
    }
    Some(FieldSet::Values(set))
}

fn parse_range_part(part: &str, min: u32, max: u32, set: &mut HashSet<u32>) -> Option<()> {
    if let Some((base, step_str)) = part.split_once('/') {
        let step: u32 = step_str.parse().ok()?;
        if step == 0 { return None; }
        let start = if base == "*" || base.is_empty() { min } else { base.parse().ok()? };
        let mut v = start;
        while v <= max {
            set.insert(v);
            v += step;
        }
    } else if part == "*" {
        for v in min..=max { set.insert(v); }
    } else if let Some((a, b)) = part.split_once('-') {
        let a: u32 = a.parse().ok()?;
        let b: u32 = b.parse().ok()?;
        for v in a..=b { set.insert(v); }
    } else {
        set.insert(part.parse().ok()?);
    }
    Some(())
}

// ── 日字段解析（支持 ? L W）──────────────────────────────────

fn parse_day_field(field: &str) -> Option<FieldSet> {
    if field == "?" {
        return Some(FieldSet::Any);
    }
    if field == "L" {
        return Some(FieldSet::LastDay);
    }
    // 检查 W 修饰符
    if let Some(day_str) = field.strip_suffix('W') {
        let day: u32 = day_str.parse().ok()?;
        if day < 1 || day > 31 { return None; }
        return Some(FieldSet::NearestWeekday(day));
    }
    parse_std_field(field, 1, 31)
}

// ── 月字段解析（支持月份名）──────────────────────────────────

fn parse_month_field(field: &str) -> Option<HashSet<u32>> {
    if field == "*" || field == "?" {
        return Some((1..=12).collect());
    }
    let mut set = HashSet::new();
    for part in field.split(',') {
        // 尝试名称替换
        let part = normalize_month_name(part);
        parse_range_part(&part, 1, 12, &mut set)?;
    }
    Some(set)
}

fn normalize_month_name(s: &str) -> String {
    match s.to_uppercase().as_str() {
        "JAN" => "1", "FEB" => "2", "MAR" => "3", "APR" => "4",
        "MAY" => "5", "JUN" => "6", "JUL" => "7", "AUG" => "8",
        "SEP" => "9", "OCT" => "10", "NOV" => "11", "DEC" => "12",
        _ => return s.to_string(),
    }.to_string()
}

// ── 周字段解析（支持 ? L # 星期名）────────────────────────────

fn parse_weekday_field(field: &str) -> Option<FieldSet> {
    if field == "?" {
        return Some(FieldSet::Any);
    }
    // 检查 # 修饰符: 2#3 = 当月第 3 个周一
    if let Some((wd_str, n_str)) = field.split_once('#') {
        let weekday = normalize_weekday_name(wd_str).parse().ok()?;
        let n: u32 = n_str.parse().ok()?;
        if weekday > 7 || n < 1 || n > 5 { return None; }
        return Some(FieldSet::NthWeekday { weekday, n });
    }
    // 检查 L 修饰符: 5L = 当月最后一个周五
    if let Some(wd_str) = field.strip_suffix('L') {
        let weekday = normalize_weekday_name(wd_str).parse().ok()?;
        if weekday > 7 { return None; }
        return Some(FieldSet::LastWeekday(weekday));
    }
    // 普通值
    parse_std_weekday_field(field)
}

fn parse_std_weekday_field(field: &str) -> Option<FieldSet> {
    if field == "*" {
        return Some(FieldSet::Any);
    }
    let mut set = HashSet::new();
    for part in field.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            let a = normalize_weekday_name(a).parse::<u32>().ok()?;
            let b = normalize_weekday_name(b).parse::<u32>().ok()?;
            for v in a..=b { set.insert(v); }
        } else if let Some((base, step_str)) = part.split_once('/') {
            let step: u32 = step_str.parse().ok()?;
            if step == 0 { return None; }
            let start = if base == "*" { 0 } else { normalize_weekday_name(base).parse().ok()? };
            let mut v = start;
            while v <= 7 { set.insert(v); v += step; }
        } else {
            set.insert(normalize_weekday_name(part).parse().ok()?);
        }
    }
    Some(FieldSet::Values(set))
}

fn normalize_weekday_name(s: &str) -> String {
    match s.to_uppercase().as_str() {
        "SUN" => "0", "MON" => "1", "TUE" => "2", "WED" => "3",
        "THU" => "4", "FRI" => "5", "SAT" => "6",
        _ => return s.to_string(),
    }.to_string()
}

// ── 日期计算辅助函数 ──────────────────────────────────────────

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    days_in_month(year, month)
}

/// 当月最后一个 weekday（0=Sun..7=Sun）
fn last_weekday_of_month(year: i32, month: u32, weekday: u32) -> Option<u32> {
    let last = days_in_month(year, month);
    let date = NaiveDate::from_ymd_opt(year, month, last)?;
    let wd = date.weekday().num_days_from_sunday();
    let wd = if wd == 0 { 7 } else { wd }; // 0=Sun → 7
    let diff = (wd as i32 - weekday as i32 + 7) % 7;
    last.checked_sub(diff as u32)
}

/// 离指定日期最近的工作日（不跨月）
fn nearest_weekday(year: i32, month: u32, day: u32) -> Option<u32> {
    let dim = days_in_month(year, month);
    let day = day.min(dim);
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let wd = date.weekday().num_days_from_sunday();
    match wd {
        0 => if day < dim { Some(day + 1) } else { Some(day - 2) }, // Sun → Mon (or Fri if month end)
        6 => if day > 1 { Some(day - 1) } else { Some(day + 2) },  // Sat → Fri (or Mon if month start)
        _ => Some(day),
    }
}

/// 当月第 n 个 weekday
fn nth_weekday(year: i32, month: u32, weekday: u32, n: u32) -> Option<u32> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let first_wd = first.weekday().num_days_from_sunday();
    let first_wd = if first_wd == 0 { 7 } else { first_wd };
    let offset = (weekday as i32 - first_wd as i32 + 7) % 7;
    let day = 1 + offset as u32 + (n - 1) * 7;
    if day <= days_in_month(year, month) { Some(day) } else { None }
}

// ── 调度任务 ───────────────────────────────────────────────────

struct ScheduledTask {
    name: String,
    cron: CronExpr,
    cron_raw: String,
    payload: String,
}

// ── 全局调度器 ──────────────────────────────────────────────────

static SCHEDULER_TX: std::sync::OnceLock<mpsc::UnboundedSender<ScheduledTask>> =
    std::sync::OnceLock::new();
static CANCEL_TX: std::sync::OnceLock<mpsc::UnboundedSender<String>> =
    std::sync::OnceLock::new();

pub fn start_scheduler() {
    let (tx, mut rx) = mpsc::unbounded_channel::<ScheduledTask>();
    let (cancel_tx, mut cancel_rx) = mpsc::unbounded_channel::<String>();
    SCHEDULER_TX.set(tx).ok();
    CANCEL_TX.set(cancel_tx).ok();

    tokio::spawn(async move {
        let mut tasks: Vec<ScheduledTask> = Vec::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!("[task] cron 调度器已启动");

        loop {
            tokio::select! {
                // 接收新任务
                Some(task) = rx.recv() => {
                    info!("[task] 添加定时任务: name={}, cron={}", task.name, task.cron_str());
                    tasks.push(task);
                }
                // 取消任务
                Some(name) = cancel_rx.recv() => {
                    let before = tasks.len();
                    tasks.retain(|t| t.name != name);
                    if tasks.len() < before {
                        info!("[task] 已取消定时任务: name={}", name);
                    } else {
                        warn!("[task] 取消任务失败（未找到）: name={}", name);
                    }
                }
                // 每秒检查
                _ = interval.tick() => {
                    let now = chrono::Local::now();
                    let sec = now.second();
                    let minute = now.minute();
                    let hour = now.hour();
                    let day = now.day();
                    let month = now.month();
                    let year = now.year();
                    let weekday = now.weekday().num_days_from_sunday();

                    for task in &tasks {
                        if task.cron.matches(sec, minute, hour, day, month, weekday, year) {
                            info!("[task] 定时任务触发: name={}", task.name);
                            let event = serde_json::json!({
                                "event": "tick",
                                "task_name": task.name,
                                "payload": task.payload,
                            });
                            if let Err(e) = bus::Bus::topic(bus::TOPIC_TASK).publish(&event.to_string()) {
                                error!("[task] 发布事件失败: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    });
}

impl ScheduledTask {
    fn cron_str(&self) -> &str {
        &self.cron_raw
    }
}

// ── 请求处理 ────────────────────────────────────────────────────

pub fn start_task_receiver() {
    // 先启动调度器
    start_scheduler();

    bus::start_topic_receiver(bus::TOPIC_TASK_MISO, |payload| async move {
        handle_task(&payload);
    });
}

fn handle_task(json: &str) {
    debug!("[task] 收到消息: {}", json);
    let Ok(req) = serde_json::from_str::<serde_json::Value>(json) else {
        warn!("[task] JSON 解析失败: {}", json);
        return;
    };

    let action = req["action"].as_str().unwrap_or("unknown");
    match action {
        "schedule" => {
            let name = req["task_name"].as_str().unwrap_or("unnamed").to_string();
            let cron_raw = req["cron"].as_str().unwrap_or("").to_string();
            let payload = req["payload"].as_str().unwrap_or("").to_string();

            let Some(cron) = CronExpr::parse(&cron_raw) else {
                warn!("[task] 无效 cron 表达式: {}", cron_raw);
                return;
            };

            let task = ScheduledTask {
                name: name.clone(),
                cron,
                cron_raw,
                payload,
            };

            if let Some(tx) = SCHEDULER_TX.get() {
                if let Err(e) = tx.send(task) {
                    error!("[task] 发送任务到调度器失败: {}", e);
                }
            } else {
                error!("[task] 调度器未初始化");
            }
        }
        "cancel" => {
            let name = req["task_name"].as_str().unwrap_or("").to_string();
            if name.is_empty() {
                warn!("[task] cancel 缺少 task_name");
                return;
            }
            if let Some(tx) = CANCEL_TX.get() {
                if let Err(e) = tx.send(name) {
                    error!("[task] 发送取消请求失败: {}", e);
                }
            }
        }
        "tick" => {
            // 事件来自调度器，插件端处理，宿主忽略
        }
        _ => {
            debug!("[task] 未知 action: {}", action);
        }
    }
}
