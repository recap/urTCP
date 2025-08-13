use std::time::Duration;
use tokio::time::{self, Interval};

pub struct TimerWheel {
    tick: Interval,
}

impl TimerWheel {
    pub fn new() -> Self {
        Self {
            tick: time::interval(Duration::from_millis(50)),
        }
    }
    pub async fn tick(&mut self) {
        self.tick.tick().await;
    }
}
