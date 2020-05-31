extern crate backoff;
extern crate reqwest;

use backoff::{Error, ExponentialBackoff, Operation};

use std::io::Read;

fn fetch_url(url: &str) -> Result<String, Error<reqwest::Error>> {
    let mut op = || {
        println!("Fetching {}", url);
        let mut resp = reqwest::blocking::get(url)?;

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
