use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;

use crate::backoff::Backoff;
use crate::error::Error;

pub trait AnyFnMut {
    type Output;
    fn call(&mut self) -> Self::Output;
}

impl<F, T> AnyFnMut for F
where
    F: FnMut() -> T,
{
    type Output = T;
    fn call(&mut self) -> Self::Output {
        self()
    }
}

/// AsyncOperation is an async operation that can be retried if it fails.
///
/// [`AsyncOperation`]: backoff/trait.AsyncOperation.html#tymethod.next_backoff
/// [`retry`]: backoff/trait.AsyncOperation.html#tymethod.retry
/// [`retry_notify`]: backoff/trait.AsyncOperation.html#tymethod.retry_notify
///
/// AsyncOperation is an async operation that can be retried if it fails.
#[async_trait]
pub trait AsyncOperation<T, E>
where
    E: Sync + Send,
{
    /// call_op implements the effective operation.
    async fn call_op(&mut self) -> Result<T, Error<E>>
    where
        T: 'async_trait,
        E: 'async_trait;

    /// Retries this operation according to the backoff policy.
    /// backoff is reset before it is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use backoff::{ExponentialBackoff, AsyncOperation, Error};
    /// async fn f() -> Result<(), Error<&'static str>> {
    ///     // Business logic...
    ///     // Give up.
    ///     Err(Error::Permanent("error"))
    /// };
    ///
    /// # async fn main_task() {
    /// let mut backoff = ExponentialBackoff::default();
    /// let _ = f.retry(&mut backoff).await.err().unwrap();
    /// # }
    ///
    /// # fn main() {
    /// #    async_std::task::block_on(main_task());
    /// # }
    /// ```
    ///
    /// or using `Box::pin` if params are required
    ///
    /// ```rust
    /// # use backoff::{ExponentialBackoff, AsyncOperation, Error};
    /// # use futures::future::BoxFuture;
    /// # async fn main_task() {
    /// let arg = "Async all the things!";
    /// let mut f = || -> BoxFuture<Result<(), Error<&'static str>>> { Box::pin(async move {
    ///     // Business logic...
    ///     println!("{}", arg);
    ///     // Give up.
    ///     Err(Error::Permanent("error"))
    /// })};
    ///
    /// let mut backoff = ExponentialBackoff::default();
    /// let _ = f.retry(&mut backoff).await.err().unwrap();
    /// # }
    ///
    /// # fn main() {
    /// #    async_std::task::block_on(main_task());
    /// # }
    /// ```
    async fn retry<B>(&mut self, backoff: &mut B) -> Result<T, Error<E>>
    where
        B: Backoff + Sync + Send,
        T: 'async_trait,
        E: 'async_trait,
    {
        let nop = |_, _| Box::pin(async {});
        self.retry_notify(backoff, nop).await
    }

    /// Retries this operation according to the backoff policy.
    /// Calls notify on failed attempts (in case of transient errors).
    /// backoff is reset before it is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use backoff::{AsyncOperation, Error};
    /// # use backoff::backoff::Stop;
    /// # use std::time::Duration;
    /// # use std::fmt::Display;
    /// async fn notify<E>(err: E, dur: Duration)
    /// where
    ///     E: Display
    /// {
    ///     println!("Error happened at {:?}: {}", dur, err);
    /// }
    ///
    /// async fn f() -> Result<(), Error<&'static str>> {
    ///     // Business logic...
    ///     Err(Error::Transient("error"))
    /// }
    ///
    /// # async fn main_task() {
    /// let mut backoff = Stop{};
    /// let _ = f.retry_notify(&mut backoff, notify).await.err().unwrap();
    /// # }
    ///
    /// # fn main() {
    /// #    async_std::task::block_on(main_task());
    /// # }
    /// ```
    async fn retry_notify<B, N>(&mut self, backoff: &mut B, mut notify: N) -> Result<T, Error<E>>
    where
        B: Backoff + Send,
        T: 'async_trait,
        E: 'async_trait,
        N: AsyncNotify<E> + Send,
    {
        backoff.reset();

        loop {
            let err = match self.call_op().await {
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

            notify.notify(err, next).await;
            async_std::task::sleep(next).await;
        }
    }
}

#[async_trait]
impl<T, E, F> AsyncOperation<T, E> for F
where
    E: Sync + Send,
    F: AnyFnMut + Sync + Send,
    F::Output: Future<Output = Result<T, Error<E>>> + Send,
{
    async fn call_op(&mut self) -> Result<T, Error<E>>
    where
        T: 'async_trait,
        E: 'async_trait,
    {
        self.call().await
    }
}

pub trait Notifier<E> {
    type Output;
    fn call(&mut self, err: E, duration: Duration) -> Self::Output;
}

impl<F, E, T> Notifier<E> for F
where
    F: FnMut(E, Duration) -> T,
{
    type Output = T;
    fn call(&mut self, err: E, duration: Duration) -> Self::Output {
        self(err, duration)
    }
}

/// AsyncNotify is called in [`retry_notify`](trait.AsyncOperation.html#method.retry_notify) in case of errors.
#[async_trait]
pub trait AsyncNotify<E> {
    async fn notify(&mut self, err: E, duration: Duration)
    where
        E: 'async_trait;
}

#[async_trait]
impl<E, F> AsyncNotify<E> for F
where
    E: Sync + Send,
    F: Notifier<E> + Sync + Send,
    F::Output: Future<Output = ()> + Send,
{
    async fn notify(&mut self, err: E, duration: Duration)
    where
        E: 'async_trait,
    {
        self.call(err, duration).await
    }
}
