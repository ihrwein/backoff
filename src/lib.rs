//! `ExponentialBackoff` is a backoff implementation that increases the backoff
//! period for each retry attempt using a randomization function that grows exponentially.
//!
//! [`next_backoff`]: backoff/trait.Backoff.html#tymethod.next_backoff
//! [`reset`]: backoff/trait.Backoff.html#tymethod.reset
//!
//! [`next_backoff`] is calculated using the following formula:
//!
//!```ignore
//!  randomized interval =
//!      retry_interval * (random value in range [1 - randomization_factor, 1 + randomization_factor])
//!```
//!
//! In other words [`next_backoff`] will range between the randomization factor
//! percentage below and above the retry interval.
//!
//! For example, given the following parameters:
//!
//!```ignore
//!retry_interval = 2
//!randomization_factor = 0.5
//!multiplier = 2
//!```
//!
//! the actual backoff period used in the next retry attempt will range between 1 and 3 seconds,
//! multiplied by the exponential, that is, between 2 and 6 seconds.
//!
//! **Note**: `max_interval` caps the `retry_interval` and not the randomized interval.
//!
//! If the time elapsed since an [`ExponentialBackoff`](type.ExponentialBackoff.html) instance is created goes past the
//! `max_elapsed_time`, then the method [`next_backoff`] starts returning `None`.
//!
//! The elapsed time can be reset by calling [`reset`].
//!
//! Example: Given the following default arguments, for 10 tries the sequence will be,
//! and assuming we go over the `max_elapsed_time` on the 10th try:
//!
//!   Request # | `retry_interval` (seconds) |  Randomized Interval (seconds)
//!  -----------|--------------------------|--------------------------------
//!    1        |  0.5                     | [0.25,   0.75]
//!    2        |  0.75                    | [0.375,  1.125]
//!    3        |  1.125                   | [0.562,  1.687]
//!    4        |  1.687                   | [0.8435, 2.53]
//!    5        |  2.53                    | [1.265,  3.795]
//!    6        |  3.795                   | [1.897,  5.692]
//!    7        |  5.692                   | [2.846,  8.538]
//!    8        |  8.538                   | [4.269, 12.807]
//!    9        | 12.807                   | [6.403, 19.210]
//!   10        | 19.210                   | None

#[cfg(feature = "async-std")]
extern crate async_std_1 as async_std;
#[cfg(feature = "tokio")]
extern crate tokio_02 as tokio;

pub mod backoff;
mod clock;
pub mod default;
mod error;
pub mod exponential;
mod retry;

pub use crate::clock::{Clock, SystemClock};
pub use crate::error::Error;
pub use crate::retry::{Notify, Operation};

#[cfg(any(feature = "tokio", feature = "async-std"))]
pub use crate::retry::r#async::future;

/// Exponential backoff policy with system's clock.
///
/// This type is preferred over
/// `exponential::ExponentialBackoff` as it is generic over any [Clocks](trait.Clock.html)
/// and in the real world mostly system's clock is used.
pub type ExponentialBackoff = exponential::ExponentialBackoff<SystemClock>;
