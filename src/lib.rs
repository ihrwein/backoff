extern crate rand;

mod error;
mod retry;
pub mod backoff;
pub mod exponential;
pub mod default;
mod clock;

pub use error::Error;
pub use clock::{Clock, SystemClock};
pub use retry::{Notify, Operation};

pub type ExponentialBackOff = exponential::ExponentialBackOff<SystemClock>;