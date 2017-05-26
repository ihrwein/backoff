extern crate rand;

pub mod error;
mod retry;
pub mod backoff;
pub mod exponential;
pub mod default;
mod clock;

pub use clock::{Clock, SystemClock};
pub use retry::{Notify, Operation};

pub type ExponentialBackOff = exponential::ExponentialBackOff<SystemClock>;