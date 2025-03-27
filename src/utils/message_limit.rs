use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct MessageLimit {
    name: String,
    current_time: f64,
    last_triggered: f64,
}

impl MessageLimit {
    pub fn new(name: &str) -> Self {
        MessageLimit {
            name: name.to_string(),
            current_time: 0.0,
            last_triggered: 0.0,
        }
    }

    pub fn check(&mut self, interval: f64) -> bool {
        self.current_time = Self::get_current_time();
        if self.current_time - self.last_triggered >= interval {
            return true;
        }
        false
    }

    pub fn handle(&mut self) {
        // 更新最后触发时间
        self.last_triggered = self.current_time;
    }

    fn get_current_time() -> f64 {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_epoch.as_secs_f64()
    }
}