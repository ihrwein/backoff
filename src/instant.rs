#[cfg(feature = "instant")]
pub use instant::Instant;

#[cfg(all(feature = "tokio_1", not(feature = "instant")))]
pub use tokio_1::time::Instant;

#[cfg(not(any(feature = "tokio_1", feature = "instant")))]
pub use std::time::Instant;
