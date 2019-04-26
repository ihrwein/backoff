# backoff

Exponential backoff and retry.

Inspired by the retry mechanism in Google's [google-http-java-client](https://github.com/google/google-http-java-client) library and
its [Golang port](https://github.com/cenkalti/backoff).

[![Build Status](https://travis-ci.org/ihrwein/backoff.svg?branch=master)](https://travis-ci.org/ihrwein/backoff)
[![crates.io](http://meritbadge.herokuapp.com/backoff)](https://crates.io/crates/backoff)

Documentation: https://docs.rs/backoff

Compile with feature `wasm-bindgen` or `stdweb` for use in WASM environments. The `Operation` trait's default implementation of `retry_notify` is not yet supported, as it uses `std::thread::sleep`.

## Usage

Just wrap your fallible operation into a closure, and call `retry` on it:

```rust
let mut op = || {
    println!("Fetching {}", url);
    let mut resp = reqwest::get(url)?;
    ...
};

let mut backoff = ExponentialBackoff::default();
op.retry(&mut backoff)
```

The retry policy will use jitters according to the `randomization_factor` field of `ExponentialBackoff`. Check the documentation for more parameters.

## Examples

### Permanent errors

Permanent errors are not retried. You have to wrap your error value explicitly
into `Error::Permanent`. You can use `Result`'s `map_err` method.

`examples/permanent_error.rs`:

```rust
extern crate backoff;
extern crate reqwest;

use backoff::{Error, ExponentialBackoff, Operation};
use reqwest::IntoUrl;

use std::fmt::Display;
use std::io::{self, Read};

fn new_io_err<E: Display>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

fn fetch_url(url: &str) -> Result<String, Error<io::Error>> {
    let mut op = || {
        println!("Fetching {}", url);
        let url = url.into_url()
            .map_err(new_io_err)
            // Permanent errors need to be explicitly constucted.
            .map_err(Error::Permanent)?;

        let mut resp = reqwest::get(url)
            // Transient errors can be constructed with the ? operator
            // or with the try! macro. No explicit conversion needed
            // from E: Error to backoff::Error;
            .map_err(new_io_err)?;

        let mut content = String::new();
        let _ = resp.read_to_string(&mut content);
        Ok(content)
    };

    let mut backoff = ExponentialBackoff::default();
    op.retry(&mut backoff)
}

fn main() {
    match fetch_url("https::///wrong URL") {
        Ok(_) => println!("Sucessfully fetched"),
        Err(err) => panic!("Failed to fetch: {}", err),
    }
}
```

Output:

```
$ time cargo run --example permanent_error
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
     Running `target/debug/examples/permanent_error`
Fetching https::///wrong URL
thread 'main' panicked at 'Failed to fetch: empty host', examples/permanent_error.rs:33
note: Run with `RUST_BACKTRACE=1` for a backtrace.

real	0m0.151s
user	0m0.116s
sys	0m0.028s
```

### Transient errors

Transient errors can be constructed by wrapping your error value into `Error::Transient`.
By using the ? operator or the `try!` macro, you always get transient errors.

`examples/retry.rs`:

```rust
extern crate backoff;
extern crate reqwest;

use backoff::{Error, ExponentialBackoff, Operation};

use std::io::Read;

fn fetch_url(url: &str) -> Result<String, Error<reqwest::Error>> {
    let mut op = || {
        println!("Fetching {}", url);
        let mut resp = reqwest::get(url)?;

        let mut content = String::new();
        let _ = resp.read_to_string(&mut content);
        Ok(content)
    };

    let mut backoff = ExponentialBackoff::default();
    op.retry(&mut backoff)
}

fn main() {
    match fetch_url("https://www.rust-lang.org") {
        Ok(_) => println!("Sucessfully fetched"),
        Err(err) => panic!("Failed to fetch: {}", err),
    }
}
```

Output with internet connection:

```
$ time cargo run --example retry
   Compiling backoff v0.1.0 (file:///home/tibi/workspace/backoff)
    Finished dev [unoptimized + debuginfo] target(s) in 1.54 secs
     Running `target/debug/examples/retry`
Fetching https://www.rust-lang.org
Sucessfully fetched

real	0m2.003s
user	0m1.536s
sys	0m0.184s
```

Output without internet connection

```
$ time cargo run --example retry
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
     Running `target/debug/examples/retry`
Fetching https://www.rust-lang.org
Fetching https://www.rust-lang.org
Fetching https://www.rust-lang.org
Fetching https://www.rust-lang.org
^C

real	0m2.826s
user	0m0.008s
sys	0m0.000s
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the Work by You, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
