use std::{borrow::Borrow, rc::Rc, ops::Deref};

mod error;
mod logic;
mod module;
mod signal;

pub use error::*;
pub use logic::*;
pub use module::*;
pub use signal::*;

pub type SignalWidth = u16;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct SignalKey(Rc<String>);

impl SignalKey {
    pub fn new(s: Rc<String>) -> Self {
        return SignalKey(s);
    }
}

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

impl From<Rc<String>> for SignalKey {
    fn from(s: Rc<String>) -> Self {
        return SignalKey(s);
    }
}

impl Borrow<str> for SignalKey {
    fn borrow(&self) -> &str {
        return self.0.as_str();
    }
}

impl Deref for SignalKey {
    type Target = Rc<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
