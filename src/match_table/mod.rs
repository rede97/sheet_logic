mod constant;
mod signal_map;

use super::excel::Sheet;
use super::parser::{self, match_cmd, match_content};
use super::utils::binary_format;
use super::verilog_model::{
    LogicElem, LogicTree, Module, Signal, SignalKey, SignalSource, Wire, WireIndex,
};
use constant::*;
use signal_map::*;
use std::collections::HashMap;
use std::ops::{Range, Shr};

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
    Segment(SignalKey),
    Flag(String),
    Primary(String),
}

#[allow(dead_code)]
pub struct MatchTable {
    header: Vec<MatchTableColumn>,
    signal_case: SignalMapCase,
    constant_case: SegsConstantCase,
    // (cidx, <signal-key, ridx>)
    flags: Vec<HashMap<String, Vec<usize>>>,
    // <ridx, signal-key>
    primary: HashMap<usize, String>,
}

#[allow(dead_code)]
impl MatchTable {
    fn gen_constant_condition(&mut self, model: &mut Module) -> HashMap<usize, Vec<SignalKey>> {
        let mut constant_condition_map: HashMap<usize, Vec<SignalKey>> = HashMap::new();
        let segs_set = &self.constant_case.segs_set;
        for (cidx, seg_case) in segs_set {
            let column = &self.header[*cidx];
            if let MatchTableColumn::Segment(signal_key) = column {
                let signal = model.get_signals()[signal_key].clone();
                let signal_len = signal.length;
                let signal_unit: LogicTree = LogicElem::Unit(signal.into()).into();
                let mut constants: Vec<u128> = seg_case.0.iter().map(|a| *a.0).collect();
                constants.sort();
                for constant in constants {
                    let constant_unit: LogicTree =
                        LogicElem::Unit(Wire::bit(signal_len, constant)).into();
                    let constant_consdition_statement = signal_unit.clone().equal(constant_unit);
                    let constant_consdition_key: SignalKey = format!(
                        "{}_is_{}",
                        signal_key.as_str(),
                        binary_format(signal_len, constant)
                    )
                    .into();
                    let ridxs = &seg_case.0[&constant];
                    println!("{} {:?}", constant_consdition_key.as_str(), ridxs);
                    ridxs
                        .iter()
                        .for_each(|ridx| match constant_condition_map.get_mut(ridx) {
                            Some(conditions) => conditions.push(constant_consdition_key.clone()),
                            None => {
                                constant_condition_map
                                    .insert(*ridx, vec![constant_consdition_key.clone()]);
                            }
                        });
                    let constant_consdition_signal = Signal::new(
                        constant_consdition_key,
                        1,
                        SignalSource::Logic(constant_consdition_statement),
                    );
                    model.add_signal(constant_consdition_signal);
                }
            } else {
                unreachable!("{} -> {:?}", *cidx, column);
            }
        }
        return constant_condition_map;
    }

    fn gen_primary(
        &self,
        model: &mut Module,
        constant_condition_map: HashMap<usize, Vec<SignalKey>>,
    ) {
        let mut primary_ridx: Vec<usize> = self.primary.iter().map(|p| *p.0).collect();
        primary_ridx.sort();
        for ridx in primary_ridx {
            let primary_signal = &self.primary[&ridx];
            let primary_signal_key: SignalKey = primary_signal.to_owned().into();
            let conditions = constant_condition_map.get(&ridx).unwrap();
            println!("{} {} -> {:?}", ridx, primary_signal, conditions);
            let mut conditions = conditions.iter().rev();
            let primary_signal_statement: LogicTree = match conditions.next() {
                Some(consition_signal_key) => {
                    let signals = model.get_signals();
                    let statement: LogicTree =
                        LogicElem::Unit(signals[consition_signal_key].clone().into()).into();
                    conditions.fold(statement, |s, cond| {
                        s.logic_and(
                            LogicElem::Unit(signals[consition_signal_key].clone().into()).into(),
                        )
                    })
                }
                None => LogicElem::Unit(Wire::bit(1, 1)).into(),
            };
        }
    }

    fn gen_flags(&self, model: &Module) {
        for flag in &self.flags {
            for (signal, ridxs) in flag {
                let signal_key: SignalKey = signal.to_owned().into();
                println!("{} -> {:?}", signal_key.as_str(), ridxs);
            }
        }
    }

    pub fn parse(model: &mut Module, sheet: &Sheet, begin: usize, end: usize) {
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

        let mut match_header: Vec<MatchTableColumn> = Vec::new();
        // header index, <ridx, signal-key>
        let mut match_primary: Option<(usize, HashMap<usize, String>)> = None;
        // [(header index, <signal-key, [ridx]>)]
        let mut match_flags: Vec<(usize, HashMap<String, Vec<usize>>)> = Vec::new();
        let ridx = row.next().unwrap();
        for cidx in sheet.row(ridx) {
            let signals = model.get_signals_mut();
            match sheet.content(ridx, cidx) {
                Some((c, None)) => {
                    if c.starts_with("#") {
                        let (_, colum_cmd) = match_cmd(c.as_str()).unwrap();
                        match &colum_cmd {
                            MatchTableColumn::Flag(_) => {
                                match_flags.push((cidx, HashMap::new()));
                            }
                            MatchTableColumn::Primary(_) => {
                                match_primary = Some((cidx, HashMap::new()));
                            }
                            _ => {}
                        }
                        match_header.push(colum_cmd);
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
                        signals.insert(seg_key.clone(), seg_signal.clone());
                        match_header.push(MatchTableColumn::Segment(seg_key));
                    }
                }
                _ => {}
            }
        }

        println!(
            "header: {:?}, primary: {:?} flags: {:?}",
            match_header, match_primary, match_flags
        );

        let mut constant_case = SegsConstantCase::new();
        let mut signal_case = SignalMapCase::new();

        for ridx in row {
            let mut row_iter = sheet.row(ridx);
            while let Some(cidx) = row_iter.next() {
                // let cidx = *cidx;
                match sheet.content(ridx, cidx) {
                    Some((content, merged)) => {
                        let (_, content) = match_content(content.as_str()).expect(content.as_str());
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
                                            merged_col.fold(0_u16, |sum, idx| match &match_header
                                                [idx]
                                            {
                                                MatchTableColumn::Segment(signal) => {
                                                    sum + model.get_signals()[signal].length
                                                }
                                                _ => {
                                                    unreachable!();
                                                }
                                            });
                                        if merged.offset.col == merged.size.col {
                                            assert_eq!(offset, constant_width);
                                        }

                                        constant.shr((constant_width - offset) as usize) as u128
                                    }
                                    None => {
                                        match &match_header[cidx] {
                                            MatchTableColumn::Segment(signal) => {
                                                assert_eq!(
                                                    model.get_signals()[signal].length,
                                                    constant_width
                                                );
                                            }
                                            _ => unreachable!(),
                                        }
                                        constant
                                    }
                                };
                                constant_case.insert(ridx - begin, cidx, constant);
                            }
                            MatchTableContent::Signal(signal, ranges) => {
                                match &match_header[cidx] {
                                    MatchTableColumn::Segment(_) => {
                                        let ranges = ranges.unwrap_or_else(|| {
                                            vec![(
                                                model
                                                    .get_signals()
                                                    .get(signal)
                                                    .expect(signal)
                                                    .length
                                                    - 1,
                                                0,
                                            )]
                                        });
                                        let signal =
                                            model.get_signals().get(signal).expect(signal).clone();
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
                                    MatchTableColumn::Primary(prefix) => {
                                        if let Some(primary) = &mut match_primary {
                                            primary.1.insert(
                                                ridx - begin,
                                                format!("{}_{}", prefix, signal),
                                            );
                                        } else {
                                            unreachable!();
                                        }
                                    }

                                    MatchTableColumn::Flag(_) => {
                                        let flags_idx = match_flags
                                            .iter()
                                            .position(|v| v.0 == cidx)
                                            .expect(format!("{}", cidx).as_str());
                                        let flag = &mut match_flags[flags_idx].1;
                                        match flag.get_mut(signal) {
                                            Some(ridxs) => {
                                                ridxs.push(ridx - begin);
                                            }
                                            None => {
                                                flag.insert(signal.into(), vec![ridx - begin]);
                                            }
                                        }
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

        let mut flags = Vec::new();
        for (cidx, flag) in match_flags {
            if let MatchTableColumn::Flag(prefix) = &match_header[cidx] {
                let mut flag_signal: HashMap<String, Vec<usize>> = HashMap::new();
                for (signal, ridxs) in flag {
                    flag_signal.insert(format!("{}_{}", prefix, signal), ridxs);
                }
                flags.push(flag_signal);
            } else {
                unreachable!();
            }
        }
        let primary = match_primary.unwrap().1;

        let mut match_table = MatchTable {
            header: match_header,
            signal_case,
            constant_case,
            flags,
            primary,
        };

        let condition_map = match_table.gen_constant_condition(model);
        match_table.gen_primary(model, condition_map);
        // match_table.gen_flags(model);
    }
}
