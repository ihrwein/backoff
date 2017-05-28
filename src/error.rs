use std::fmt;
use std::error;

pub enum Error<P, T> {
    Permanent(P),
    Transient(T),
}

impl<P, T> fmt::Display for Error<P, T>
    where P: fmt::Display,
          T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Permanent(ref err) => err.fmt(f),
            Error::Transient(ref err) => err.fmt(f),
        }
    }
}

impl<P, T> fmt::Debug for Error<P, T>
    where P: fmt::Debug,
          T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (name, err) = match *self {
            Error::Permanent(ref err) => ("Permanent", err as &fmt::Debug),
            Error::Transient(ref err) => ("Transient", err as &fmt::Debug),
        };
        f.debug_tuple(name).field(err).finish()
    }
}

impl<P, T> error::Error for Error<P, T>
    where P: error::Error,
          T: error::Error
{
    fn description(&self) -> &str {
        match *self {
            Error::Permanent(_) => "permanent error",
            Error::Transient(_) => "transient error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Permanent(ref err) => err.cause(),
            Error::Transient(ref err) => err.cause(),
        }
    }
}
