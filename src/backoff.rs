use std::time::Duration;

pub trait Backoff {
    fn reset(&mut self) {}
    fn next_backoff(&mut self) -> Option<Duration>;
}

pub struct Zero {}

impl Backoff for Zero {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(Duration::default())
    }
}

pub struct Stop {}

impl Backoff for Stop {
    fn next_backoff(&mut self) -> Option<Duration> {
        None
    }
}

pub struct Constant {
    interval: Duration,
}

impl Constant {
    pub fn new(interval: Duration) -> Constant {
        Constant { interval: interval }
    }
}

impl Backoff for Constant {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}
