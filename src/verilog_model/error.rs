use std::fmt;

use super::SignalWidth;

#[derive(Debug)]
pub enum Error {
    SignalIndexOutOfRange { len: SignalWidth, idx: SignalWidth },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

impl std::error::Error for Error {}
