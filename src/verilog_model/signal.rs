use super::{Error, LogicTree, SignalKey, SignalWidth};

#[derive(Clone)]
#[allow(unused)]
pub enum Wire {
    Single {
        signal: SignalKey,
        idx: SignalWidth,
    },
    Range {
        signal: SignalKey,
        h: SignalWidth,
        l: SignalWidth,
    },
    Multiple {
        signal: SignalKey,
        idxs: Vec<SignalWidth>,
    },
}

impl From<&Signal> for Wire {
    fn from(s: &Signal) -> Wire {
        return s.range(s.length - 1, 0).unwrap();
    }
}

#[derive(Clone)]
#[allow(unused)]
pub enum SignalSource {
    Unconnected,
    Input,
    Wire(Wire),
    Logic(LogicTree),
}

impl Default for SignalSource {
    fn default() -> Self {
        SignalSource::Unconnected
    }
}

pub struct Signal {
    pub key: SignalKey,
    pub length: SignalWidth,
    pub from: SignalSource,
}

#[allow(dead_code)]
impl Signal {
    pub fn new(key: SignalKey, length: SignalWidth, from: SignalSource) -> Signal {
        return Signal { key, length, from };
    }

    fn assert_index(&self, idx: SignalWidth) -> Option<Error> {
        if idx >= self.length {
            return Some(Error::SignalIndexOutOfRange {
                len: self.length,
                idx: idx,
            });
        }
        return None;
    }

    pub fn single(&self, idx: SignalWidth) -> Result<Wire, Error> {
        if let Some(e) = self.assert_index(idx) {
            return Err(e);
        }
        return Ok(Wire::Single {
            signal: self.key.clone(),
            idx: idx,
        });
    }

    pub fn range(&self, h: SignalWidth, l: SignalWidth) -> Result<Wire, Error> {
        if let Some(e) = self.assert_index(h).or_else(|| self.assert_index(l)) {
            return Err(e);
        }
        return Ok(Wire::Range {
            signal: self.key.clone(),
            h,
            l,
        });
    }

    pub fn multiple(&self, idxs: Vec<SignalWidth>) -> Result<Wire, Error> {
        if let Some(e) = idxs.iter().find_map(|&idx| self.assert_index(idx)) {
            return Err(e);
        }
        return Ok(Wire::Multiple {
            signal: self.key.clone(),
            idxs,
        });
    }
}
