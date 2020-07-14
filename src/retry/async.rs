use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project::pin_project;
#[cfg(feature = "tokio")]
use tokio::time::{delay_for, Delay};

use crate::{backoff::Backoff, error::Error};

use super::{NoopNotify, Notify};

#[cfg(feature = "async-std")]
type Delay = Pin<Box<dyn Future<Output = ()> + 'static + Send>>;

#[cfg(feature = "async-std")]
fn delay_for(duration: std::time::Duration) -> Delay {
    Box::pin(async_std::task::sleep(duration))
}

pub mod future {
    use super::*;

    /// Retries given `operation` according to the [`Backoff`] policy.
    /// [`Backoff`] is reset before it is used.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "async-std")]
    /// # extern crate async_std_1 as async_std;
    /// # #[cfg(feature = "tokio")]
    /// # extern crate tokio_02 as tokio;
    /// use backoff::{future, ExponentialBackoff};
    ///
    /// async fn f() -> Result<(), backoff::Error<&'static str>> {
    ///     // Business logic...
    ///     Err(backoff::Error::Permanent("error"))
    /// }
    ///
    /// # #[cfg_attr(feature = "async-std", async_std::main)]
    /// # #[cfg_attr(feature = "tokio", tokio::main)]
    /// # async fn main() {
    /// future::retry(ExponentialBackoff::default(), f).await.err().unwrap();
    /// # }
    /// ```
    pub fn retry<I, E, Fn, B>(backoff: B, operation: Fn) -> Retry<B, NoopNotify, Fn, Fn::Fut>
    where
        B: Backoff,
        Fn: FutureOperation<I, E>,
    {
        retry_notify(backoff, operation, NoopNotify)
    }

    /// Retries given `operation` according to the [`Backoff`] policy.
    /// Calls `notify` on failed attempts (in case of [`Error::Transient`]).
    /// [`Backoff`] is reset before it is used.
    ///
    /// # Async `notify`
    ///
    /// `notify` can be neither `async fn` or [`Future`]. If you need to perform some async
    /// operations inside `notify`, consider to use `tokio::spawn` or `async_std::task::spawn`
    /// for that.
    ///
    /// The reason behind this is that [`Retry`] future cannot be responsible for polling
    /// `notify` future, because can easily be dropped _before_ `notify` is completed.
    /// So, considering the fact that most of the time no async operations are required in
    /// `notify`, it's up to the caller to decide how async `notify` should be performed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "async-std")]
    /// # extern crate async_std_1 as async_std;
    /// # #[cfg(feature = "tokio")]
    /// # extern crate tokio_02 as tokio;
    /// use backoff::{future, backoff::Stop};
    ///
    /// async fn f() -> Result<(), backoff::Error<&'static str>> {
    ///     // Business logic...
    ///     Err(backoff::Error::Transient("error"))
    /// }
    ///
    /// # #[cfg_attr(feature = "async-std", async_std::main)]
    /// # #[cfg_attr(feature = "tokio", tokio::main)]
    /// # async fn main() {
    /// future::retry_notify(Stop {}, f, |e, dur| println!("Error happened at {:?}: {}", dur, e))
    ///     .await
    ///     .err()
    ///     .unwrap();
    /// # }
    /// ```
    pub fn retry_notify<I, E, Fn, B, N>(
        mut backoff: B,
        mut operation: Fn,
        notify: N,
    ) -> Retry<B, N, Fn, Fn::Fut>
    where
        B: Backoff,
        Fn: FutureOperation<I, E>,
        N: Notify<E>,
    {
        backoff.reset();
        let fut = operation.call_op();
        Retry {
            backoff,
            delay: None,
            operation,
            fut,
            notify,
        }
    }

    /// [`FutureOperation`] is a [`Future`] operation that can be retried if it fails with the
    /// provided [`Backoff`].
    ///
    /// Note, that this should not be a [`Future`] itself, but rather something producing a
    /// [`Future`] (a closure, for example).
    pub trait FutureOperation<I, E> {
        /// Type of [`Future`] that this [`FutureOperation`] produces.
        type Fut: Future<Output = Result<I, Error<E>>>;

        /// Calls this [`FutureOperation`] returning a [`Future`] to be executed.
        fn call_op(&mut self) -> Self::Fut;

        /// Retries this [`FutureOperation`] according to the [`Backoff`] policy.
        /// [`Backoff`] is reset before it is used.
        ///
        /// # Example
        ///
        /// ```rust
        /// # #[cfg(feature = "async-std")]
        /// # extern crate async_std_1 as async_std;
        /// # #[cfg(feature = "tokio")]
        /// # extern crate tokio_02 as tokio;
        /// use backoff::{future::FutureOperation as _, ExponentialBackoff};
        ///
        /// async fn f() -> Result<(), backoff::Error<&'static str>> {
        ///     // Business logic...
        ///     Err(backoff::Error::Permanent("error"))
        /// }
        ///
        /// # #[cfg_attr(feature = "async-std", async_std::main)]
        /// # #[cfg_attr(feature = "tokio", tokio::main)]
        /// # async fn main() {
        /// f.retry(ExponentialBackoff::default()).await.err().unwrap();
        /// # }
        /// ```
        fn retry<B>(self, backoff: B) -> Retry<B, NoopNotify, Self, Self::Fut>
        where
            B: Backoff,
            Self: Sized,
        {
            retry(backoff, self)
        }

        /// Retries this [`FutureOperation`] according to the [`Backoff`] policy.
        /// Calls `notify` on failed attempts (in case of [`Error::Transient`]).
        /// [`Backoff`] is reset before it is used.
        ///
        /// # Async `notify`
        ///
        /// `notify` can be neither `async fn` or [`Future`]. If you need to perform some async
        /// operations inside `notify`, consider to use `tokio::spawn` or `async_std::task::spawn`
        /// for that.
        ///
        /// The reason behind this is that [`Retry`] future cannot be responsible for polling
        /// `notify` future, because can easily be dropped _before_ `notify` is completed.
        /// So, considering the fact that most of the time no async operations are required in
        /// `notify`, it's up to the caller to decide how async `notify` should be performed.
        ///
        /// # Example
        ///
        /// ```rust
        /// # #[cfg(feature = "async-std")]
        /// # extern crate async_std_1 as async_std;
        /// # #[cfg(feature = "tokio")]
        /// # extern crate tokio_02 as tokio;
        /// use backoff::{future::FutureOperation as _, backoff::Stop};
        ///
        /// async fn f() -> Result<(), backoff::Error<&'static str>> {
        ///     // Business logic...
        ///     Err(backoff::Error::Transient("error"))
        /// }
        ///
        /// # #[cfg_attr(feature = "async-std", async_std::main)]
        /// # #[cfg_attr(feature = "tokio", tokio::main)]
        /// # async fn main() {
        /// f.retry_notify(Stop {}, |e, dur| println!("Error happened at {:?}: {}", dur, e))
        ///     .await
        ///     .err()
        ///     .unwrap();
        /// # }
        /// ```
        fn retry_notify<B, N>(self, backoff: B, notify: N) -> Retry<B, N, Self, Self::Fut>
        where
            B: Backoff,
            N: Notify<E>,
            Self: Sized,
        {
            retry_notify(backoff, self, notify)
        }
    }

    impl<I, E, Fn, Fut> FutureOperation<I, E> for Fn
    where
        Fn: FnMut() -> Fut,
        Fut: Future<Output = Result<I, Error<E>>>,
    {
        type Fut = Fut;

        fn call_op(&mut self) -> Self::Fut {
            self()
        }
    }
}

/// Retry implementation.
#[pin_project]
pub struct Retry<B, N, Fn, Fut> {
    /// [`Backoff`] implementation to count next [`Retry::delay`] with.
    backoff: B,

    /// [`Future`] which delays execution before next [`Retry::operation`] invocation.
    delay: Option<Delay>,

    /// Operation to be retried. It must return [`Future`].
    operation: Fn,

    /// [`Future`] being resolved once [`Retry::operation`] is completed.
    #[pin]
    fut: Fut,

    /// [`Notify`] implementation to track [`Retry`] ticks.
    notify: N,
}

impl<B, N, Fn, Fut, I, E> Future for Retry<B, N, Fn, Fut>
where
    B: Backoff,
    N: Notify<E>,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
{
    type Output = Result<I, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            if this.delay.is_some() {
                ready!(Pin::new(this.delay.as_mut().unwrap()).poll(cx));
                let _ = this.delay.take();
            }

            match ready!(this.fut.as_mut().poll(cx)) {
                Ok(v) => return Poll::Ready(Ok(v)),
                Err(Error::Permanent(e)) => return Poll::Ready(Err(e)),
                Err(Error::Transient(e)) => match this.backoff.next_backoff() {
                    Some(duration) => {
                        this.notify.notify(e, duration);
                        this.delay.replace(delay_for(duration));
                        this.fut.set((this.operation)());
                    }
                    None => return Poll::Ready(Err(e)),
                },
            }
        }
    }
}
