extern crate rand;

pub mod error;
pub mod retry;
pub mod backoff;
pub mod exponential;
pub mod default;
mod clock;

pub type ExponentialBackOff = exponential::ExponentialBackOff<SystemClock>;
pub use clock::{Clock, SystemClock};