use std::fmt;
use std::error;

pub enum Error<E> {
    Permanent(E),
    Transient(E),
}

impl<E> fmt::Display for Error<E>
    where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Permanent(ref err) => err.fmt(f),
            Error::Transient(ref err) => err.fmt(f),
        }
    }
}

impl<E> fmt::Debug for Error<E>
    where E: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (name, err) = match *self {
            Error::Permanent(ref err) => ("Permanent", err as &fmt::Debug),
            Error::Transient(ref err) => ("Transient", err as &fmt::Debug),
        };
        f.debug_tuple(name).field(err).finish()
    }
}

impl<E> error::Error for Error<E>
    where E: error::Error
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
