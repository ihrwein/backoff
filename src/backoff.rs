use std::time::Duration;

/// `Backoff` is a backoff policy for retrying an operation.
pub trait Backoff {
    /// Resets the internal state to the initial value.
    fn reset(&mut self) {}
    /// next_backoff() time is elapsed before it is called again.
    /// If it returns None, it means the operation timed out and no
    /// further retries are done.
    fn next_backoff(&mut self) -> Option<Duration>;
}

impl<B: Backoff + ?Sized> Backoff for Box<B> {
    fn next_backoff(&mut self) -> Option<Duration> {
        let this: &mut B = self;
        this.next_backoff()
    }

    fn reset(&mut self) {
        let this: &mut B = self;
        this.reset()
    }
}

/// Immediately retry the operation.
#[derive(Debug)]
pub struct Zero {}

impl Backoff for Zero {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(Duration::default())
    }
}

/// The operation should never be retried.
#[derive(Debug)]
pub struct Stop {}

impl Backoff for Stop {
    fn next_backoff(&mut self) -> Option<Duration> {
        None
    }
}

/// Constant is a backoff policy which always returns
/// a constant duration.
#[derive(Debug)]
pub struct Constant {
    interval: Duration,
}

impl Constant {
    /// Creates a new Constant backoff with `interval` constant
    /// backoff.
    pub fn new(interval: Duration) -> Constant {
        Constant { interval }
    }
}

impl Backoff for Constant {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}

/// Backoff policy with a fixed number of retries with a constant interval.
#[derive(Debug)]
pub struct FixedNumber {
    interval: Duration,
    max_attempts: usize,
    current_attempt: usize,
}

impl FixedNumber {
    /// Creates a new FixedNumber backoff with fixed number of retry attempts (`max_attempts`)
    /// and constant duration between them (`interval`)
    pub fn new(interval: Duration, max_attempts: usize) -> Self {
        Self {
            interval,
            max_attempts,
            current_attempt: 0,
        }
    }
}

impl Backoff for FixedNumber {
    fn reset(&mut self) {
        self.current_attempt = 0;
    }

    fn next_backoff(&mut self) -> Option<Duration> {
        if self.current_attempt < self.max_attempts - 1 {
            self.current_attempt += 1;
            Some(self.interval)
        } else {
            None
        }
    }
}
