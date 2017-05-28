use std::time::Duration;
use std::thread;

use error::Error;
use backoff::BackOff;

pub trait Operation<P, T> {
    fn call_op(&self) -> Result<(), Error<P, T>>;
}

impl<P, T, F> Operation<P, T> for F
    where F: Fn() -> Result<(), Error<P, T>>
{
    fn call_op(&self) -> Result<(), Error<P, T>> {
        self()
    }
}

pub trait Notify<E> {
    fn notify(&self, err: E, duration: Duration);
}

impl<E, F> Notify<E> for F
    where F: Fn(E, Duration)
{
    fn notify(&self, err: E, duration: Duration) {
        self(err, duration)
    }
}

pub fn retry<O, B, P, T>(operation: O, backoff: B) -> Result<(), Error<P, T>>
    where O: Operation<P, T>,
          B: BackOff
{
    let nop = |_, _| ();
    retry_notify(operation, backoff, nop)
}

pub fn retry_notify<O, B, N, P, T>(operation: O,
                                   mut backoff: B,
                                   notify: N)
                                   -> Result<(), Error<P, T>>
    where O: Operation<P, T>,
          N: Notify<T>,
          B: BackOff
{
    backoff.reset();

    loop {
        let err = match operation.call_op() {
            Ok(_) => return Ok(()),
            Err(err) => err,
        };

        let err = match err {
            Error::Permanent(err) => return Err(Error::Permanent(err)),
            Error::Transient(err) => err,
        };

        let next = match backoff.next_back_off() {
            Some(next) => next,
            None => return Err(Error::Transient(err)),
        };

        notify.notify(err, next);
        thread::sleep(next);
    }
}
