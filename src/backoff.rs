use std::time::Duration;

pub trait BackOff {
    fn reset(&mut self) {}
    fn next_back_off(&mut self) -> Option<Duration>;
}

pub struct Zero {}

impl BackOff for Zero {
    fn next_back_off(&mut self) -> Option<Duration> {
        Some(Duration::default())
    }
}

pub struct Stop {}

impl BackOff for Stop {
    fn next_back_off(&mut self) -> Option<Duration> {
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

impl BackOff for Constant {
    fn next_back_off(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}
