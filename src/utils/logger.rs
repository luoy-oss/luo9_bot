use std::collections::VecDeque;
use std::io::Write;
use std::sync::Mutex;
use tracing_subscriber::fmt;
use tracing_subscriber::filter::EnvFilter;

/// 全局日志缓冲区，存储最近的日志行供 WebUI 读取
pub static LOG_BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

const MAX_LOG_LINES: usize = 500;

/// 向缓冲区追加一行日志
pub fn push_log(line: String) {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        if buf.len() >= MAX_LOG_LINES {
            buf.pop_front();
        }
        buf.push_back(line);
    }
}

/// 获取日志缓冲区快照
pub fn get_logs() -> Vec<String> {
    LOG_BUFFER.lock().map(|buf| buf.iter().cloned().collect()).unwrap_or_default()
}

/// 去除 ANSI 转义序列
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // 跳过整个转义序列：\x1b[ ... m
            for next in chars.by_ref() {
                if next == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 同时写入 stdout 和全局缓冲区的 writer
struct GlobalWriter;

impl Write for GlobalWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 缓冲区存储纯文本（去掉 ANSI 颜色码）
        let text = String::from_utf8_lossy(buf);
        push_log(strip_ansi(&text));
        // 终端输出带颜色的原始内容
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

/// 实现 MakeWriter，让 tracing-subscriber 使用 GlobalWriter
struct GlobalWriterMaker;

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for GlobalWriterMaker {
    type Writer = GlobalWriter;

    fn make_writer(&'a self) -> Self::Writer {
        GlobalWriter
    }
}

pub fn init(level: &str) {
    let mut filter = EnvFilter::new(level);
    // 屏蔽第三方网络库的 debug 日志
    for noisy in ["reqwest", "h2", "hyper_util", "rustls_platform_verifier", "rustls"] {
        filter = filter.add_directive(format!("{}=warn", noisy).parse().unwrap());
    }

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .with_writer(GlobalWriterMaker)
        .init();

    tracing::info!("日志系统已初始化，级别: {}", level);
}
