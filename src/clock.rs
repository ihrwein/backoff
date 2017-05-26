use std::time::Instant;

pub trait Clock {
    fn now(&self) -> Instant;
}

pub struct SystemClock {}

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        SystemClock{}
    }
}

