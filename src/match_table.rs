use super::excel::Sheet;
use super::verilog_model::{Module, Signal, SignalKey, SignalSource, Wire, WireIndex};
use std::collections::{HashMap, HashSet};
use std::ops::{Range, Shr};

use super::parser::{self, match_cmd, match_content};

pub enum Section {
    None,
    Match(usize),
}

#[derive(Debug)]
pub enum MatchTableContent<'a> {
    Constant(u16, u128),
    WireCase(&'a str, Option<Vec<(u16, u16)>>),
}

#[derive(Debug)]
pub enum MatchTableColumn {
    None,
    Segment(Signal),
    Primary(String),
    Flag(String),
}

pub struct ConstantCase(HashMap<usize, HashSet<u64>>);

impl std::fmt::Display for ConstantCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<&usize> = self.0.keys().collect();
        keys.sort();
        for k in keys {
            let mut constants: Vec<&u64> = self.0[k].iter().collect();
            constants.sort();
            write!(f, "{}: {:?}; ", k, constants)?;
        }
        return Ok(());
    }
}

#[derive(Debug)]
struct SignalMapSlot {
    ranges: Vec<(u16, u16)>,
    segs: Range<usize>,
}

#[derive(Debug)]
struct SignalMap {
    slots: HashMap<usize, Vec<SignalMapSlot>>,
}

#[allow(dead_code)]
pub struct MatchTable {
    target: Wire,
    header: Vec<MatchTableColumn>,
    signal_case: HashMap<SignalKey, Vec<(u16, Wire)>>,
    constant_case: ConstantCase,
}

#[allow(dead_code)]
impl MatchTable {
    pub fn new(model: &mut Module, sheet: &Sheet, begin: usize, end: usize) {
        let mut row = begin..end;
        let ridx = row.next().unwrap();

        let target_signal_str = sheet.content(ridx, 1).and_then(|(c, _)| Some(c)).unwrap();
        let (_, (signal_name, ranges)) =
            parser::sginal_ref(&target_signal_str).expect(&target_signal_str);
        let target_signal = model.get_signals().get(signal_name).unwrap().clone();

        let target = match ranges {
            Some(ranges) => Wire::Multiple {
                signal: target_signal.key.clone(),
                idxs: ranges.iter().map(|&(h, l)| WireIndex::new(h, l)).collect(),
            },
            None => Wire::Independent {
                signal: target_signal.key.clone(),
                idx: (0..target_signal.length).into(),
            },
        };

        let match_signal = Signal::new(
            format!("match_{}", target).into(),
            target.len(),
            SignalSource::Wire(target),
        );
        model.add_signal(match_signal.clone());

        let mut header: Vec<MatchTableColumn> = Vec::new();
        let ridx = row.next().unwrap();
        for cidx in sheet.row(ridx) {
            let mdl = model.get_signals_mut();
            match sheet.content(ridx, cidx) {
                Some((c, None)) => {
                    if c.starts_with("#") {
                        let (_, columd_cmd) = match_cmd(c.as_str()).unwrap();
                        header.push(columd_cmd);
                    } else {
                        let (_, ((h, l), alias)) = parser::range_alias(&c).unwrap();
                        let seg_wire = match_signal.range(l..h + 1).unwrap();
                        let seg_key: SignalKey = alias
                            .unwrap_or(format!("{}_{}to{}", target_signal.key.as_str(), h, l))
                            .into();
                        let seg_signal = Signal::new(
                            seg_key.clone(),
                            seg_wire.len(),
                            SignalSource::Wire(seg_wire),
                        );
                        mdl.insert(seg_key.clone(), seg_signal.clone());
                        header.push(MatchTableColumn::Segment(seg_signal));
                    }
                }
                _ => {}
            }
        }

        println!("header: {:?}", header);

        let mut content_case: HashMap<usize, HashSet<u64>> = HashMap::new();
        let mut signal_case: HashMap<SignalKey, SignalMap> = HashMap::new();

        for ridx in row {
            let mut row_iter = sheet.row(ridx);
            while let Some(cidx) = row_iter.next() {
                // let cidx = *cidx;
                match sheet.content(ridx, cidx) {
                    Some((c, merged)) => {
                        let (_, content) = match_content(c.as_str()).expect(c.as_str());
                        match content {
                            MatchTableContent::Constant(constant_width, constant) => {
                                println!("@ [{}:{}]", ridx, cidx,);
                                let constant = match merged.and_then(|merged| {
                                    if merged.size.col == 1 {
                                        None
                                    } else {
                                        Some(merged)
                                    }
                                }) {
                                    Some(merged) => {
                                        let merged_col =
                                            (cidx - merged.offset.col as usize)..cidx + 1;
                                        let offset =
                                            merged_col.fold(0_u16, |sum, idx| match &header[idx] {
                                                MatchTableColumn::Segment(seg) => sum + seg.length,
                                                _ => {
                                                    unreachable!();
                                                }
                                            });
                                        if merged.offset.col == merged.size.col {
                                            assert_eq!(offset, constant_width);
                                        }

                                        constant.shr((constant_width - offset) as usize) as u64
                                    }
                                    None => {
                                        match &header[cidx] {
                                            MatchTableColumn::Segment(s) => {
                                                assert_eq!(s.length, constant_width);
                                            }
                                            _ => unreachable!(),
                                        }
                                        constant as u64
                                    }
                                };

                                if !content_case.contains_key(&cidx) {
                                    content_case.insert(cidx, HashSet::new());
                                }
                                content_case.get_mut(&cidx).unwrap().insert(constant as u64);
                            }
                            MatchTableContent::WireCase(s, ranges) => {
                                println!("{} {:?}", s, ranges);
                                match &header[cidx] {
                                    MatchTableColumn::Segment(_) => {
                                        let ranges = ranges.unwrap_or_else(|| {
                                            vec![(
                                                model.get_signals().get(s).expect(s).length - 1,
                                                0,
                                            )]
                                        });
                                        let signal = model.get_signals().get(s).expect(s).clone();
                                        let signal_map = match signal_case.get_mut(&signal.key) {
                                            Some(w) => w,
                                            None => {
                                                signal_case.insert(
                                                    signal.key.clone(),
                                                    SignalMap {
                                                        slots: HashMap::new(),
                                                    },
                                                );
                                                signal_case.get_mut(&signal.key).unwrap()
                                            }
                                        };
                                        let merged_cols = merged
                                            .and_then(|m| Some(m.size.col as usize))
                                            .unwrap_or(1);
                                        for _ in 0..merged_cols - 1 {
                                            row_iter.next();
                                        }

                                        let slots = match signal_map.slots.get_mut(&ridx) {
                                            Some(slots) => slots,
                                            None => {
                                                signal_map.slots.insert(ridx, Vec::new());
                                                signal_map.slots.get_mut(&ridx).unwrap()
                                            }
                                        };
                                        slots.push(SignalMapSlot {
                                            ranges,
                                            segs: cidx..cidx + merged_cols,
                                        });
                                    }

                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        println!("constant {}", ConstantCase(content_case));
        println!("signal map {:?}", signal_case);
        // return MatchTable { target, segments };
    }
}
