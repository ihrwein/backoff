extern crate tokio_02 as tokio;

use backoff::{future::FutureOperation as _, ExponentialBackoff};

async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    (|| async {
        println!("Fetching {}", url);
        Ok(reqwest::get(url).await?.text().await?)
    })
    .retry(ExponentialBackoff::default())
    .await
}

#[tokio::main]
async fn main() {
    match fetch_url("https://www.rust-lang.org").await {
        Ok(_) => println!("Successfully fetched"),
        Err(err) => panic!("Failed to fetch: {}", err),
    }
}
