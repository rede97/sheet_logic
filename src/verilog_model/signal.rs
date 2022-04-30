use super::{Error, LogicTree, SignalKey, SignalWidth};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct WireIndex(Range<SignalWidth>);

impl WireIndex {
    pub fn new(h: SignalWidth, l: SignalWidth) -> Self {
        return WireIndex(l..h + 1);
    }
}

impl From<Range<SignalWidth>> for WireIndex {
    fn from(range: Range<SignalWidth>) -> Self {
        WireIndex(range)
    }
}

impl std::fmt::Display for WireIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let range = &self.0;
        if range.len() == 1 {
            return write!(f, "{}", &range.start);
        } else {
            return write!(f, "{}to{}", &range.end - 1, &range.start);
        }
    }
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum Wire {
    Constant(u128, SignalWidth),
    Independent {
        signal: SignalKey,
        idx: WireIndex,
    },
    Multiple {
        signal: SignalKey,
        idxs: Vec<WireIndex>,
    },
    Compose {
        wires: Vec<Wire>,
    },
}

impl From<Signal> for Wire {
    fn from(s: Signal) -> Wire {
        return s.range(0..s.length).unwrap();
    }
}

impl std::fmt::Display for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Wire::Constant(_, w) => {
                return write!(f, "bit_{}", *w);
            }
            Wire::Independent { signal: key, idx } => {
                return write!(f, "{}_{}", key.as_str(), idx);
            }
            Wire::Multiple { signal: key, idxs } => {
                write!(f, "{}", key.as_str())?;
                idxs.iter().for_each(|idx| {
                    write!(f, "_{}", idx).unwrap();
                })
            }
            Wire::Compose { wires } => match wires.len() {
                0 => {
                    unreachable!()
                }
                1 => {
                    return wires[0].fmt(f);
                }
                _ => {
                    write!(f, "wires")?;
                    wires.iter().for_each(|w| {
                        write!(f, "_{}", w).unwrap();
                    });
                }
            },
        }
        return Ok(());
    }
}

impl Wire {
    pub fn bit(w: SignalWidth, c: u128) -> Self {
        return Wire::Constant(c, w);
    }
    pub fn compose(wires: Vec<Wire>) -> Self {
        return Wire::Compose { wires };
    }

    pub fn len(&self) -> SignalWidth {
        match &self {
            Wire::Constant(_, w) => {
                return *w;
            }
            Wire::Independent { signal: _, idx } => {
                return idx.0.len() as SignalWidth;
            }
            Wire::Multiple { signal: _, idxs } => {
                return idxs
                    .iter()
                    .fold(0, |len, idx| len + idx.0.len() as SignalWidth);
            }
            Wire::Compose { wires } => {
                return wires.iter().fold(0, |len, wire| len + wire.len());
            }
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub struct Signal {
    pub key: SignalKey,
    pub length: SignalWidth,
    pub from: SignalSource,
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.key.as_str(), self.length)?;
        return Ok(());
    }
}

impl std::fmt::Debug for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return <Self as std::fmt::Display>::fmt(&self, f);
    }
}

#[allow(dead_code)]
impl Signal {
    pub fn new(key: SignalKey, length: SignalWidth, from: SignalSource) -> Signal {
        return Signal { key, length, from };
    }

    fn assert_index(&self, idx: &WireIndex) -> Option<Error> {
        let range = &idx.0;

        if range.start >= self.length {
            return Some(Error::SignalIndexOutOfRange {
                len: self.length,
                idx: range.start,
            });
        }
        if range.end > self.length {
            return Some(Error::SignalIndexOutOfRange {
                len: self.length,
                idx: range.end,
            });
        }

        return None;
    }

    pub fn single(&self, idx: SignalWidth) -> Result<Wire, Error> {
        return self.indep(WireIndex(idx..idx + 1));
    }

    pub fn range(&self, idx: Range<SignalWidth>) -> Result<Wire, Error> {
        return self.indep(WireIndex(idx));
    }

    pub fn indep(&self, idx: WireIndex) -> Result<Wire, Error> {
        if let Some(e) = self.assert_index(&idx) {
            return Err(e);
        }
        return Ok(Wire::Independent {
            signal: self.key.clone(),
            idx,
        });
    }

    pub fn multiple(&self, idxs: Vec<WireIndex>) -> Result<Wire, Error> {
        if let Some(e) = idxs.iter().find_map(|idx| self.assert_index(&idx)) {
            return Err(e);
        }
        return Ok(Wire::Multiple {
            signal: self.key.clone(),
            idxs,
        });
    }
}
