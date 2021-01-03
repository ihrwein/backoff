#[cfg(feature = "futures")]
pub mod r#async;

use std::thread;
use std::time::Duration;

use crate::backoff::Backoff;
use crate::error::Error;

/// Retries this operation according to the backoff policy.
/// backoff is reset before it is used.
///
/// # Examples
///
/// ```rust
/// # use backoff::{ExponentialBackoff, Error, retry};
/// let f = || -> Result<(), Error<&str>> {
///     // Business logic...
///     // Give up.
///     Err(Error::Permanent("error"))
/// };
///
/// let mut backoff = ExponentialBackoff::default();
/// let _ = retry(&mut backoff, f).err().unwrap();
/// ```
pub fn retry<F, B, T, E>(backoff: &mut B, op: F) -> Result<T, Error<E>>
where
    F: FnMut() -> Result<T, Error<E>>,
    B: Backoff,
{
    let nop = |_, _| ();
    retry_notify(backoff, op, nop)
}

/// Retries this operation according to the backoff policy.
/// Calls notify on failed attempts (in case of transient errors).
/// backoff is reset before it is used.
///
/// # Examples
///
/// ```rust
/// # use backoff::{Error, retry_notify};
/// # use backoff::backoff::Stop;
/// # use std::time::Duration;
/// let notify = |err, dur| { println!("Error happened at {:?}: {}", dur, err); };
/// let f = || -> Result<(), Error<&str>> {
///     // Business logic...
///     Err(Error::Transient("error"))
/// };
///
/// let mut backoff = Stop{};
/// let _ = retry_notify(&mut backoff, f, notify).err().unwrap();
/// ```
pub fn retry_notify<F, B, N, T, E>(backoff: &mut B, mut op: F, mut notify: N) -> Result<T, Error<E>>
where
    F: FnMut() -> Result<T, Error<E>>,
    B: Backoff,
    N: Notify<E>,
{
    backoff.reset();

    loop {
        let err = match op() {
            Ok(v) => return Ok(v),
            Err(err) => err,
        };

        let err = match err {
            Error::Permanent(err) => return Err(Error::Permanent(err)),
            Error::Transient(err) => err,
        };

        let next = match backoff.next_backoff() {
            Some(next) => next,
            None => return Err(Error::Transient(err)),
        };

        notify.notify(err, next);
        thread::sleep(next);
    }
}

/// Notify is called in [`retry_notify`](trait.Operation.html#method.retry_notify) in case of errors.
pub trait Notify<E> {
    fn notify(&mut self, err: E, duration: Duration);
}

impl<E, F> Notify<E> for F
where
    F: FnMut(E, Duration),
{
    fn notify(&mut self, err: E, duration: Duration) {
        self(err, duration)
    }
}

/// No-op implementation of [`Notify`]. Literally does nothing.
#[derive(Debug, Clone, Copy)]
pub struct NoopNotify;

impl<E> Notify<E> for NoopNotify {
    fn notify(&mut self, _: E, _: Duration) {}
}
