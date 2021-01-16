use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::ready;
use pin_project::pin_project;

use crate::{backoff::Backoff, error::Error};

use super::{NoopNotify, Notify};

pub mod future {
    use super::*;

    pub trait Sleeper {
        type Sleep: Future<Output = ()> + Send + 'static;
        fn sleep(&self, dur: Duration) -> Self::Sleep;
    }

    /// Retries given `operation` according to the [`Backoff`] policy
    /// and waits between attempts using the passed [`Sleeper`].
    /// [`Backoff`] is reset before it is used.
    ///
    /// If you're using tokio or async_std, you may want to look at
    /// [`backoff::tokio::retry`](tokio::retry) or [`backoff::async_std::retry`](async_std::retry)
    ///
    /// # Example
    ///
    /// ```rust
    /// # struct MySleeper;
    /// # impl backoff::future::Sleeper for MySleeper {
    /// #     type Sleep = std::future::Ready<()>;
    /// #     fn sleep(&self, _dur: std::time::Duration) -> Self::Sleep { std::future::ready(()) }
    /// # }
    /// use backoff::ExponentialBackoff;
    ///
    /// async fn f() -> Result<(), backoff::Error<&'static str>> {
    ///     // Business logic...
    ///     Err(backoff::Error::Permanent("error"))
    /// }
    ///
    /// # async fn go() {
    /// backoff::future::retry(MySleeper, ExponentialBackoff::default(), f).await.err().unwrap();
    /// # }
    /// # fn main() { futures_executor::block_on(go()); }
    /// ```
    pub fn retry<S, I, E, Fn, Fut, B>(
        sleeper: S,
        backoff: B,
        operation: Fn,
    ) -> Retry<S, B, NoopNotify, Fn, Fut>
    where
        S: Sleeper,
        B: Backoff,
        Fn: FnMut() -> Fut,
        Fut: Future<Output = Result<I, Error<E>>>,
    {
        retry_notify(sleeper, backoff, operation, NoopNotify)
    }

    /// Retries given `operation` according to the [`Backoff`] policy
    /// and waits between attempts using the passed [`Sleeper`].
    /// Calls `notify` on failed attempts (in case of [`Error::Transient`]).
    /// [`Backoff`] is reset before it is used.
    ///
    /// If you're using tokio or async_std, you may want to look at
    /// [`crate::tokio::retry_notify`] or [`crate::async_std::retry_notify`]
    ///
    /// # Async `notify`
    ///
    /// `notify` can be neither `async fn` or [`Future`]. If you need to perform some async
    /// operations inside `notify`, consider using your runtimes task-spawning functionality.
    ///
    /// The reason behind this is that [`Retry`] future cannot be responsible for polling
    /// `notify` future, because can easily be dropped _before_ `notify` is completed.
    /// So, considering the fact that most of the time no async operations are required in
    /// `notify`, it's up to the caller to decide how async `notify` should be performed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # struct MySleeper;
    /// # impl backoff::future::Sleeper for MySleeper {
    /// #     type Sleep = std::future::Ready<()>;
    /// #     fn sleep(&self, _dur: std::time::Duration) -> Self::Sleep { std::future::ready(()) }
    /// # }
    /// use backoff::backoff::Stop;
    ///
    /// async fn f() -> Result<(), backoff::Error<&'static str>> {
    ///     // Business logic...
    ///     Err(backoff::Error::Transient("error"))
    /// }
    ///
    /// # async fn go() {
    /// let err = backoff::future::retry_notify(MySleeper, Stop {}, f, |e, dur| {
    ///     println!("Error happened at {:?}: {}", dur, e)
    /// })
    /// .await
    /// .err()
    /// .unwrap();
    /// assert_eq!(err, "error");
    /// # }
    /// # fn main() { futures_executor::block_on(go()); }
    /// ```
    pub fn retry_notify<S, I, E, Fn, Fut, B, N>(
        sleeper: S,
        mut backoff: B,
        mut operation: Fn,
        notify: N,
    ) -> Retry<S, B, N, Fn, Fut>
    where
        S: Sleeper,
        B: Backoff,
        Fn: FnMut() -> Fut,
        Fut: Future<Output = Result<I, Error<E>>>,
        N: Notify<E>,
    {
        backoff.reset();
        let fut = operation();
        Retry {
            sleeper,
            backoff,
            delay: OptionPinned::None,
            operation,
            fut,
            notify,
        }
    }
}

use future::Sleeper;

/// Retry implementation.
#[pin_project]
pub struct Retry<S: Sleeper, B, N, Fn, Fut> {
    /// The [`Sleeper`] that we generate the `delay` futures from.
    sleeper: S,

    /// [`Backoff`] implementation to count next [`Retry::delay`] with.
    backoff: B,

    /// [`Future`] which delays execution before next [`Retry::operation`] invocation.
    #[pin]
    delay: OptionPinned<S::Sleep>,

    /// Operation to be retried. It must return [`Future`].
    operation: Fn,

    /// [`Future`] being resolved once [`Retry::operation`] is completed.
    #[pin]
    fut: Fut,

    /// [`Notify`] implementation to track [`Retry`] ticks.
    notify: N,
}

#[pin_project(project = OptionProj)]
enum OptionPinned<T> {
    Some(#[pin] T),
    None,
}

impl<S, B, N, Fn, Fut, I, E> Future for Retry<S, B, N, Fn, Fut>
where
    S: Sleeper,
    B: Backoff,
    N: Notify<E>,
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, Error<E>>>,
{
    type Output = Result<I, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            if let OptionProj::Some(delay) = this.delay.as_mut().project() {
                ready!(delay.poll(cx));
                this.delay.set(OptionPinned::None);
            }

            match ready!(this.fut.as_mut().poll(cx)) {
                Ok(v) => return Poll::Ready(Ok(v)),
                Err(Error::Permanent(e)) => return Poll::Ready(Err(e)),
                Err(Error::Transient(e)) => match this.backoff.next_backoff() {
                    Some(duration) => {
                        this.notify.notify(e, duration);
                        this.delay
                            .set(OptionPinned::Some(this.sleeper.sleep(duration)));
                        this.fut.set((this.operation)());
                    }
                    None => return Poll::Ready(Err(e)),
                },
            }
        }
    }
}

// runtime-specific convenience modules

macro_rules! doc_comment {
    ($x:expr, $($tt:tt)*) => {
        #[doc = $x]
        $($tt)*
    };
}

macro_rules! gen_rt_module {
    ($rt:ident, $rt_crate:ident, $Sleeper:ident, feature = $feat:literal) => {
        #[cfg(feature = $feat)]
        pub mod $rt {
            use super::*;

            doc_comment! {
                concat!("Retries given `operation` according to the [`Backoff`] policy.
[`Backoff`] is reset before it is used.

The returned future can be spawned onto a ",stringify!($rt),"-compatible runtime.

# Example

```rust
# extern crate ",stringify!($rt_crate)," as ",stringify!($rt),r#";
use backoff::ExponentialBackoff;

async fn f() -> Result<(), backoff::Error<&'static str>> {
    // Business logic...
    Err(backoff::Error::Permanent("error"))
}

# #["#,stringify!($rt),"::main]
# async fn main() {
let err = backoff::",stringify!($rt),r#"::retry(ExponentialBackoff::default(), f).await.err().unwrap();
assert_eq!(err, "error");
# }
```"#),
                pub fn retry<I, E, F, Fut, B>(
                    backoff: B,
                    operation: F,
                ) -> Retry<impl Sleeper, B, NoopNotify, F, Fut>
                where
                    B: Backoff,
                    F: FnMut() -> Fut,
                    Fut: Future<Output = Result<I, Error<E>>>,
                {
                    future::retry($Sleeper, backoff, operation)
                }
            }

            doc_comment! {
                concat!("Retries given `operation` according to the [`Backoff`] policy.
Calls `notify` on failed attempts (in case of [`Error::Transient`]).
[`Backoff`] is reset before it is used.

The returned future can be spawned onto a ",stringify!($rt),"-compatible runtime.

# Async `notify`

`notify` can be neither `async fn` or [`Future`]. If you need to perform some async
operations inside `notify`, consider using `",stringify!($rt),r"::task::spawn` for that.

The reason behind this is that [`Retry`] future cannot be responsible for polling
`notify` future, because can easily be dropped _before_ `notify` is completed.
So, considering the fact that most of the time no async operations are required in
`notify`, it's up to the caller to decide how async `notify` should be performed.

# Example

```rust
# extern crate ",stringify!($rt_crate)," as ",stringify!($rt),r#";
use backoff::ExponentialBackoff;
use backoff::backoff::Stop;

async fn f() -> Result<(), backoff::Error<&'static str>> {
    // Business logic...
    Err(backoff::Error::Transient("error"))
}

# #["#,stringify!($rt),"::main]
# async fn main() {
backoff::",stringify!($rt),r#"::retry_notify(Stop {}, f, |e, dur| println!("Error happened at {:?}: {}", dur, e))
    .await
    .err()
    .unwrap();
# }
```"#),
                pub fn retry_notify<I, E, Fn, Fut, B, N>(
                    backoff: B,
                    operation: Fn,
                    notify: N,
                ) -> Retry<impl Sleeper, B, N, Fn, Fut>
                where
                    B: Backoff,
                    Fn: FnMut() -> Fut,
                    Fut: Future<Output = Result<I, Error<E>>>,
                    N: Notify<E>,
                {
                    future::retry_notify($Sleeper, backoff, operation, notify)
                }
            }
        }
    };
}

#[cfg(feature = "tokio")]
struct TokioSleeper;
#[cfg(feature = "tokio")]
impl Sleeper for TokioSleeper {
    type Sleep = ::tokio_1::time::Sleep;
    fn sleep(&self, dur: Duration) -> Self::Sleep {
        ::tokio_1::time::sleep(dur)
    }
}

gen_rt_module!(tokio, tokio_1, TokioSleeper, feature = "tokio");

#[cfg(feature = "async-std")]
struct AsyncStdSleeper;
#[cfg(feature = "async-std")]
impl Sleeper for AsyncStdSleeper {
    type Sleep = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
    fn sleep(&self, dur: Duration) -> Self::Sleep {
        Box::pin(::async_std_1::task::sleep(dur))
    }
}

gen_rt_module!(
    async_std,
    async_std_1,
    AsyncStdSleeper,
    feature = "async-std"
);
