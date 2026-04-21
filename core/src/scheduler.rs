use std::time::Duration;

pub struct Scheduler {
    interval: Duration,
}

impl Scheduler {
    pub fn new(interval_mins: u32) -> Self {
        Self {
            interval: Duration::from_secs(u64::from(interval_mins.max(1)) * 60),
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(60)
    }
}
