# backoff

Exponential backoff and retry.

Inspired by the retry mechanism in Google's [google-http-java-client](https://github.com/google/google-http-java-client) library and
its [Golang port](https://github.com/cenkalti/backoff).

[![Build Status](https://travis-ci.org/ihrwein/backoff.svg?branch=master)](https://travis-ci.org/ihrwein/backoff)
[![crates.io](http://meritbadge.herokuapp.com/backoff)](https://crates.io/crates/backoff)
[![Documentation](https://docs.rs/backoff/badge.svg)](https://docs.rs/backoff)

Compile with feature `wasm-bindgen` or `stdweb` for use in WASM environments. `retry_notify` is not yet supported, as it uses `std::thread::sleep`.

:warning: **BREAKING CHANGES**: migration instructions under [Breaking changes](#breaking-changes).

## Overview

`backoff` is small crate which allows you to retry operations according to backoff policies. It provides:

* Error type to wrap errors as either transient of permanent,
* different backoff algorithms, including exponential,
* supporting both sync and async code.

## Sync example

Just wrap your fallible operation into a closure, and pass it into `retry`:

```rust
use backoff::{retry, ExponentialBackoff, Error};

let op = || {
    reqwest::blocking::get("http://example.com").map_err(Error::Transient)
};

let _ = retry(&mut ExponentialBackoff::default(), op);
```

The retry policy will use jitters according to the `randomization_factor` field of `ExponentialBackoff`. Check the documentation for more parameters.

## Async example

Futures are supported by the `futures` module:

```rust
use backoff::ExponentialBackoff;
use backoff::tokio::retry;

async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    retry(ExponentialBackoff::default(), async {
        println!("Fetching {}", url);
        Ok(reqwest::get(url).await?.text().await?)
    })
    .await
}
```

## Breaking changes

### 0.2.x -> 0.3.x

#### Removal of Operation trait

The `Operation` trait has been removed, please use normal closures implementing `FnMut` instead. The `retry` and `retry_notify` methods were converted to free functions, available in the crate's root:  

```diff
-let mut op = || {
+let op = || {
     println!("Fetching {}", url);
     let mut resp = reqwest::get(url)?;
     ...
 };
 
 let mut backoff = ExponentialBackoff::default();
-op.retry(&mut backoff)
+retry(&mut backoff, op)
```

#### Removal of FutureOperation trait

The `FutureOperation` trait has been removed, please use normal Futures instead. There is no need to create a future which creates another future. The `retry` and `retry_notify` methods were converted to free functions, available in the crate's root:

```diff

+extern crate tokio_1 as tokio;
+
+use backoff::ExponentialBackoff;
 
 async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
-    (|| async {
+    backoff::tokio::retry(ExponentialBackoff::default(), async {
         Ok(reqwest::get(url).await?.text().await?)
     })
-    .retry(ExponentialBackoff::default())
     .await
 }
```

#### Changes in feature flags

* `stdweb` flag was removed, as the project is abandoned.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the Work by You, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
