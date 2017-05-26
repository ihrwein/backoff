extern crate rand;

pub mod error;
pub mod retry;
pub mod backoff;
pub mod exponential;
pub mod default;

pub type ExponentialBackOff = exponential::ExponentialBackOff<exponential::SystemClock>;
pub use exponential::Clock;
pub use exponential::SystemClock;