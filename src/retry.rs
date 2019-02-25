use std::time::Duration;
use std::thread;

use crate::error::Error;
use crate::backoff::Backoff;

/// Operation is an operation that can be retried if it fails.
///
/// [`Operation`]: backoff/trait.Operation.html#tymethod.next_backoff
/// [`retry`]: backoff/trait.Operation.html#tymethod.retry
/// [`retry_notify`]: backoff/trait.Operation.html#tymethod.retry_notify
///
/// Operation is an operation that can be retried if it fails.
pub trait Operation<T, E> {
    /// call_op implements the effective operation.
    fn call_op(&mut self) -> Result<T, Error<E>>;

    /// Retries this operation according to the backoff policy.
    /// backoff is reset before it is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use backoff::{ExponentialBackoff, Operation, Error};
    /// let mut f = || -> Result<(), Error<&str>> {
    ///     // Business logic...
    ///     // Give up.
    ///     Err(Error::Permanent("error"))
    /// };
    ///
    /// let mut backoff = ExponentialBackoff::default();
    /// let _ = f.retry(&mut backoff).err().unwrap();
    /// ```
    fn retry<B>(&mut self, backoff: &mut B) -> Result<T, Error<E>>
        where B: Backoff
    {
        let nop = |_, _| ();
        self.retry_notify(backoff, nop)
    }

    /// Retries this operation according to the backoff policy.
    /// Calls notify on failed attempts (in case of transient errors).
    /// backoff is reset before it is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use backoff::{Operation, Error};
    /// # use backoff::backoff::Stop;
    /// # use std::time::Duration;
    /// let notify = |err, dur| { println!("Error happened at {:?}: {}", dur, err); };
    /// let mut f = || -> Result<(), Error<&str>> {
    ///     // Business logic...
    ///     Err(Error::Transient("error"))
    /// };
    ///
    /// let mut backoff = Stop{};
    /// let _ = f.retry_notify(&mut backoff, notify).err().unwrap();
    /// ```
    fn retry_notify<B, N>(&mut self, backoff: &mut B, mut notify: N) -> Result<T, Error<E>>
        where N: Notify<E>,
              B: Backoff
    {
        backoff.reset();

        loop {
            let err = match self.call_op() {
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
}


impl<T, E, F> Operation<T, E> for F
    where F: FnMut() -> Result<T, Error<E>>
{
    fn call_op(&mut self) -> Result<T, Error<E>> {
        self()
    }
}

/// Notify is called in [`retry_notify`](trait.Operation.html#method.retry_notify) in case of errors.
pub trait Notify<E> {
    fn notify(&mut self, err: E, duration: Duration);
}

impl<E, F> Notify<E> for F
    where F: Fn(E, Duration)
{
    fn notify(&mut self, err: E, duration: Duration) {
        self(err, duration)
    }
}
