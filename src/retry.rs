use std::time::Duration;
use std::thread;

use error::Error;
use backoff::BackOff;

pub trait Operation<T, E> {
    fn call_op(&mut self) -> Result<T, Error<E>>;

    fn retry<B>(&mut self, backoff: B) -> Result<T, Error<E>>
        where B: BackOff
    {
        let nop = |_, _| ();
        self.retry_notify(backoff, nop)
    }

    fn retry_notify<B, N>(&mut self, mut backoff: B, mut notify: N) -> Result<T, Error<E>>
        where N: Notify<E>,
              B: BackOff
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

            let next = match backoff.next_back_off() {
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

pub fn simple_op<F>(f: F) -> SimpleOperation<F> {
    SimpleOperation {f : f} 
}

pub struct SimpleOperation<F> {
    f: F
}

impl<T, E, F> Operation<T, E> for SimpleOperation<F>
    where F: FnMut() -> Result<T, E>
{
    fn call_op(&mut self) -> Result<T, Error<E>> {
       (self.f)().map_err(Error::Transient)
    }
}

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
