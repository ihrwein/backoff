use std::time::Duration;
use std::time::Instant;

use rand;

pub const DEFAULT_INITIAL_INTERVAL_MILLIS: u64 = 500;
pub const DEFAULT_RANDOMIZATION_FACTOR: f64 = 0.5;
pub const DEFAULT_MULTIPLIER: f64 = 1.5;
pub const DEFAULT_MAX_INTERVAL_MILLIS: u64 = 60000;
pub const DEFAULT_MAX_ELAPSED_TIME_MILLIS: u64 = 900000;

/* */
struct ExponentialBackOff {
    current_interval: Duration,
    initial_interval: Duration,
    randomization_factor: f64,
    multiplier: f64,
    max_interval: Duration,
    max_elapsed_time: Duration,
    clock: Box<Clock>,

    start_time: Instant,
}

impl Default for ExponentialBackOff {
    fn default() -> ExponentialBackOff {
        let mut eb = ExponentialBackOff {
            current_interval: Duration::from_millis(DEFAULT_INITIAL_INTERVAL_MILLIS),
            initial_interval: Duration::from_millis(DEFAULT_INITIAL_INTERVAL_MILLIS),
            randomization_factor: DEFAULT_RANDOMIZATION_FACTOR,
            multiplier: DEFAULT_MULTIPLIER,
            max_interval: Duration::from_millis(DEFAULT_MAX_INTERVAL_MILLIS),
            max_elapsed_time: Duration::from_millis(DEFAULT_MAX_ELAPSED_TIME_MILLIS),
            clock: Box::new(SystemClock {}),
            start_time: Instant::now(),
        };
        eb.reset();
        eb
    }
}

impl ExponentialBackOff {
    pub fn get_elapsed_time(&self) -> Duration {
        self.clock.now().duration_since(self.start_time)
    }

    fn get_random_value_from_interval(randomization_factor: f64,
                                      random: f64,
                                      current_interval: Duration)
                                      -> Duration {
        let current_interval_millis = (current_interval.as_secs() * 1000_u64 +
                                       (current_interval.subsec_nanos() as u64 / 1000_0000)) as
                                      f64;

        let delta = randomization_factor * current_interval_millis;
        let min_interval = current_interval_millis - delta;
        let max_interval = current_interval_millis + delta;
        // Get a random value from the range [minInterval, maxInterval].
        // The formula used below has a +1 because if the minInterval is 1 and the maxInterval is 3 then
        // we want a 33% chance for selecting either 1, 2 or 3.
        let millis = (min_interval + (random * (max_interval - min_interval + 1.0))) as u64;
        Duration::from_millis(millis)
    }
}

trait Clock {
    fn now(&self) -> Instant;
}

struct SystemClock {}

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

use backoff::BackOff;

impl BackOff for ExponentialBackOff {
    fn reset(&mut self) {
        self.current_interval = self.initial_interval;
        self.start_time = self.clock.now();
    }

    fn next_back_off(&mut self) -> Option<Duration> {
        if self.get_elapsed_time() > self.max_elapsed_time {
            None
        } else {
            let random = rand::random::<f64>();
            let randomized_interval =
                Self::get_random_value_from_interval(self.randomization_factor,
                                                     random,
                                                     self.current_interval);
            Some(randomized_interval)
        }
    }
}
