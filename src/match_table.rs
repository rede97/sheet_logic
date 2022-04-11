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
    Signal(&'a str, Option<Vec<(u16, u16)>>),
}

#[derive(Debug)]
pub enum MatchTableColumn {
    None,
    Segment(Signal),
    Primary(String),
    Flag(String),
}

pub struct SegConstantCase(HashMap<u64, Vec<usize>>);

pub struct SegsConstantCase {
    segs_set: HashMap<usize, SegConstantCase>,
}

pub struct ConstantCasePrinter<'a> {
    cc: &'a SegsConstantCase,
    header: &'a Vec<MatchTableColumn>,
}

impl<'a> ConstantCasePrinter<'a> {
    pub fn new(cc: &'a SegsConstantCase, header: &'a Vec<MatchTableColumn>) -> Self {
        return ConstantCasePrinter { cc, header };
    }
}

impl<'a> std::fmt::Display for ConstantCasePrinter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<&usize> = self.cc.segs_set.keys().collect();
        keys.sort();
        for k in keys {
            let signal = match &self.header[*k] {
                MatchTableColumn::Segment(seg) => &seg.key,
                _ => unreachable!(),
            };
            let mut constants: Vec<(&u64, &Vec<usize>)> = self.cc.segs_set[k].0.iter().collect();
            constants.sort_by(|a, b| a.0.cmp(b.0));
            writeln!(f, "{}: {:?}", signal.as_str(), constants)?;
        }
        return Ok(());
    }
}

impl SegsConstantCase {
    pub fn new() -> Self {
        return SegsConstantCase {
            segs_set: HashMap::new(),
        };
    }

    pub fn insert(&mut self, case_idx: usize, seg_idx: usize, constant: u64) {
        if !self.segs_set.contains_key(&seg_idx) {
            self.segs_set
                .insert(seg_idx, SegConstantCase(HashMap::new()));
        }

        let case = self.segs_set.get_mut(&seg_idx).unwrap();
        match case.0.get_mut(&constant) {
            Some(v) => {
                v.push(case_idx);
            }
            None => {
                let v = vec![case_idx];
                case.0.insert(constant, v);
            }
        }

        // .insert(constant as u64);
    }
}

#[derive(Debug)]
struct SignalMapSlot {
    ranges: Vec<(u16, u16)>,
    segs: Range<usize>,
}

#[derive(Debug)]
struct SignalMapSlots {
    slots: Vec<SignalMapSlot>,
    slot_case: HashMap<usize, Vec<usize>>,
}

impl SignalMapSlots {
    pub fn new() -> Self {
        return {
            SignalMapSlots {
                slots: Vec::new(),
                slot_case: HashMap::new(),
            }
        };
    }

    pub fn insert(&mut self, case_idx: usize, ranges: Vec<(u16, u16)>, segs: Range<usize>) {
        let slot_idx = match self
            .slots
            .iter()
            .position(|s| s.ranges == ranges && s.segs == segs)
        {
            Some(slot_idx) => slot_idx,
            None => {
                let slot_idx = self.slots.len();
                self.slots.push(SignalMapSlot {
                    ranges: ranges,
                    segs: segs,
                });
                slot_idx
            }
        };

        match self.slot_case.get_mut(&case_idx) {
            Some(v) => {
                v.push(slot_idx);
            }
            None => {
                let v = vec![slot_idx];
                self.slot_case.insert(case_idx, v);
            }
        }
    }
}

#[derive(Debug)]
struct SignalMapCase(HashMap<SignalKey, SignalMapSlots>);

impl SignalMapCase {
    pub fn new() -> Self {
        return SignalMapCase(HashMap::new());
    }

    pub fn insert(
        &mut self,
        signal_key: &SignalKey,
        case_idx: usize,
        ranges: Vec<(u16, u16)>,
        segs: Range<usize>,
    ) {
        if !self.0.contains_key(signal_key) {
            self.0.insert(signal_key.clone(), SignalMapSlots::new());
        }
        self.0
            .get_mut(signal_key)
            .unwrap()
            .insert(case_idx, ranges, segs);
    }
}

#[allow(dead_code)]
pub struct MatchTable {
    target: Wire,
    header: Vec<MatchTableColumn>,
    signal_case: SignalMapSlots,
    constant_case: SegsConstantCase,
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
        let mut primary_idx = None;
        let ridx = row.next().unwrap();
        for cidx in sheet.row(ridx) {
            let mdl = model.get_signals_mut();
            match sheet.content(ridx, cidx) {
                Some((c, None)) => {
                    if c.starts_with("#") {
                        let (_, colum_cmd) = match_cmd(c.as_str()).unwrap();
                        match &colum_cmd {
                            MatchTableColumn::Primary(_) => {
                                primary_idx = Some(cidx);
                            }
                            _ => {}
                        }
                        header.push(colum_cmd);
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

        let primary_idx = primary_idx.unwrap();
        println!("header: {:?}, primary: {}", header, primary_idx);

        let mut constant_case = SegsConstantCase::new();
        let mut signal_case = SignalMapCase::new();
        let mut primary_case: HashMap<usize, SignalKey> = HashMap::new();

        for ridx in row {
            let mut row_iter = sheet.row(ridx);
            while let Some(cidx) = row_iter.next() {
                // let cidx = *cidx;
                match sheet.content(ridx, cidx) {
                    Some((c, merged)) => {
                        let (_, content) = match_content(c.as_str()).expect(c.as_str());
                        match content {
                            MatchTableContent::Constant(constant_width, constant) => {
                                // println!("@ [{}:{}]", ridx, cidx,);
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
                                constant_case.insert(ridx - begin, cidx, constant);
                            }
                            MatchTableContent::Signal(s, ranges) => {
                                // println!("{} {:?}", s, ranges);
                                match &header[cidx] {
                                    MatchTableColumn::Segment(_) => {
                                        let ranges = ranges.unwrap_or_else(|| {
                                            vec![(
                                                model.get_signals().get(s).expect(s).length - 1,
                                                0,
                                            )]
                                        });
                                        let signal = model.get_signals().get(s).expect(s).clone();
                                        let merged_cols = merged
                                            .and_then(|m| Some(m.size.col as usize))
                                            .unwrap_or(1);
                                        for _ in 0..merged_cols - 1 {
                                            row_iter.next();
                                        }

                                        signal_case.insert(
                                            &signal.key,
                                            ridx - begin,
                                            ranges,
                                            cidx..cidx + merged_cols,
                                        );
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
        println!(
            "constant case:\n{}",
            ConstantCasePrinter::new(&constant_case, &header)
        );
        println!("signal case\n{:?}", signal_case);
        // return MatchTable { target, segments };
    }
}
