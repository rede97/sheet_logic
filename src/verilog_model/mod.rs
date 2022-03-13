use std::{borrow::Borrow, rc::Rc};

mod error;
mod module;
mod signal;
mod logic;

pub use error::*;
pub use signal::*;
pub use logic::*;

type SignalWidth = u16;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SignalKey(Rc<String>);

impl From<&str> for SignalKey {
    fn from(s: &str) -> Self {
        return SignalKey(Rc::new(s.into()));
    }
}

impl From<String> for SignalKey {
    fn from(s: String) -> Self {
        return SignalKey(Rc::new(s));
    }
}

impl Borrow<str> for SignalKey {
    fn borrow(&self) -> &str {
        return self.0.as_str();
    }
}
