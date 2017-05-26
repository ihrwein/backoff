use std::time::Duration;
use std::time::Instant;

use rand;

pub const DEFAULT_INITIAL_INTERVAL_MILLIS: u64 = 500;
pub const DEFAULT_RANDOMIZATION_FACTOR: f64 = 0.5;
pub const DEFAULT_MULTIPLIER: f64 = 1.5;
pub const DEFAULT_MAX_INTERVAL_MILLIS: u64 = 60000;
pub const DEFAULT_MAX_ELAPSED_TIME_MILLIS: u64 = 900000;

/* */
pub struct ExponentialBackOff<C> {
    pub current_interval: Duration,
    pub initial_interval: Duration,
    pub randomization_factor: f64,
    pub multiplier: f64,
    pub max_interval: Duration,
    pub max_elapsed_time: Option<Duration>,
    pub clock: C,
    pub start_time: Instant,
}

impl<C> Default for ExponentialBackOff<C> where C: Clock + Default {
    fn default() -> ExponentialBackOff<C> {
        let mut eb = ExponentialBackOff {
            current_interval: Duration::from_millis(DEFAULT_INITIAL_INTERVAL_MILLIS),
            initial_interval: Duration::from_millis(DEFAULT_INITIAL_INTERVAL_MILLIS),
            randomization_factor: DEFAULT_RANDOMIZATION_FACTOR,
            multiplier: DEFAULT_MULTIPLIER,
            max_interval: Duration::from_millis(DEFAULT_MAX_INTERVAL_MILLIS),
            max_elapsed_time: Some(Duration::from_millis(DEFAULT_MAX_ELAPSED_TIME_MILLIS)),
            clock: C::default(),
            start_time: Instant::now(),
        };
        eb.reset();
        eb
    }
}

impl<C: Clock> ExponentialBackOff<C> {
    pub fn get_elapsed_time(&self) -> Duration {
        self.clock.now().duration_since(self.start_time)
    }

    fn get_random_value_from_interval(randomization_factor: f64,
                                      random: f64,
                                      current_interval: Duration)
                                      -> Duration {
        let current_interval_nanos = duration_to_nanos(current_interval);

        let delta = randomization_factor * current_interval_nanos;
        let min_interval = current_interval_nanos - delta;
        let max_interval = current_interval_nanos + delta;
        // Get a random value from the range [minInterval, maxInterval].
        // The formula used below has a +1 because if the minInterval is 1 and the maxInterval is 3 then
        // we want a 33% chance for selecting either 1, 2 or 3.
        let diff = max_interval - min_interval;
        let nanos = min_interval + (random * (diff + 1.0));
        nanos_to_duration(nanos)
    }

    fn increment_current_interval(&mut self) -> Duration {
        let current_interval_nanos = duration_to_nanos(self.current_interval);
        let max_interval_nanos = duration_to_nanos(self.max_interval);
        // Check for overflow, if overflow is detected set the current interval to the max interval.
        if current_interval_nanos >= max_interval_nanos / self.multiplier {
            self.max_interval
        } else {
            let nanos = current_interval_nanos * self.multiplier;
            nanos_to_duration(nanos)
        }
    }
}

fn duration_to_nanos(d: Duration) -> f64 {
    d.as_secs() as f64 * 1000_000_000.0 + d.subsec_nanos() as f64
}

fn nanos_to_duration(nanos: f64) -> Duration {
    let secs = nanos / 1000_000_000.0;
    let nanos = nanos as u64 % 1000_000_000;
    Duration::new(secs as u64, nanos as u32)
}

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

use backoff::BackOff;

impl<C> BackOff for ExponentialBackOff<C> where C: Clock {
    fn reset(&mut self) {
        self.current_interval = self.initial_interval;
        self.start_time = self.clock.now();
    }

    fn next_back_off(&mut self) -> Option<Duration> {
        match self.max_elapsed_time {
            Some(v) if self.get_elapsed_time() > v => None,
            _ => {
                let random = rand::random::<f64>();
                let randomized_interval =
                    Self::get_random_value_from_interval(self.randomization_factor,
                                                        random,
                                                        self.current_interval);
                self.current_interval = self.increment_current_interval();
                Some(randomized_interval)
            }
        }
    }
}

#[test]
fn get_randomized_interval() {
    // 33% chance of being 1.
    let f = ExponentialBackOff::<SystemClock>::get_random_value_from_interval;
    assert_eq!(Duration::new(0, 1), f(0.5, 0.0, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 1), f(0.5, 0.33, Duration::new(0, 2)));
    // 33% chance of being 2.
    assert_eq!(Duration::new(0, 2), f(0.5, 0.34, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 2), f(0.5, 0.66, Duration::new(0, 2)));
    // 33% chance of being 3.
    assert_eq!(Duration::new(0, 3), f(0.5, 0.67, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 3), f(0.5, 0.99, Duration::new(0, 2)));
}
