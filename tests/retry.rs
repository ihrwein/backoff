extern crate backoff;

use backoff::Error;
use backoff::ExponentialBackoff;

use std::io;

#[test]
fn retry() {
    let mut i = 0;
    let success_on = 3;

    {
        let f = || -> Result<(), Error<io::Error>> {
            i += 1;
            if i == success_on {
                return Ok(());
            }

            Err(Error::Transient(io::Error::new(
                io::ErrorKind::Other,
                "err",
            )))
        };

        let mut backoff = ExponentialBackoff::default();
        backoff::retry(&mut backoff, f).ok().unwrap();
    }

    assert_eq!(i, success_on);
}

#[test]
fn permanent_error_immediately_returned() {
    let f = || -> Result<(), Error<io::Error>> {
        Err(Error::Permanent(io::Error::new(
            io::ErrorKind::Other,
            "err",
        )))
    };

    let mut backoff = ExponentialBackoff::default();
    match backoff::retry(&mut backoff, f).err().unwrap() {
        Error::Permanent(_) => (),
        other => panic!("{}", other),
    }
}
