mod excel;
mod parser;
#[allow(dead_code)]
mod verilog_model;
use excel::{Cell, CellPosition, Sheet};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use verilog_model::{Module, Signal, SignalKey, SignalSource, Wire};

use parser::{match_cmd, match_content};
use verilog_model::WireIndex;

enum Section {
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

#[allow(dead_code)]
struct MatchTable {
    target: Wire,
    header: Vec<MatchTableColumn>,
    signal_case: HashMap<SignalKey, Vec<(u16, Wire)>>,
    constant_case: HashMap<usize, HashSet<u64>>,
}

#[allow(dead_code)]
impl MatchTable {
    pub fn new(model: &mut Module, sheet: &Sheet, begin: usize, end: usize) {
        let row = &mut sheet.cells.iter().skip(begin).take(end - begin).enumerate();
        let (ridx, header) = row.next().unwrap();
        let target_signal_str = sheet
            .content(begin + ridx, 1)
            .and_then(|(c, _)| Some(c))
            .unwrap();
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
        let (ridx, headers_row) = row.next().unwrap();
        for (cidx, col) in headers_row.iter().enumerate() {
            let mdl = model.get_signals_mut();
            match sheet.content(begin + ridx, cidx) {
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

        let mut content_case: HashMap<usize, HashSet<u64>> = HashMap::new();

        for (ridx, content_row) in row {
            for (cidx, cell) in content_row.iter().enumerate() {
                println!("-> {:?} {}:{}", cell, ridx, cidx);
                match sheet.content(begin + ridx, cidx) {
                    Some((c, off)) => {
                        let (_, content) = match_content(c.as_str()).expect(c.as_str());
                        match content {
                            MatchTableContent::Constant(w, c) => match &header[cidx] {
                                MatchTableColumn::Segment(s) => match off {
                                    Some(_off) => {}
                                    None => {
                                        assert_eq!(s.length, w);
                                        if !content_case.contains_key(&cidx) {
                                            content_case.insert(cidx, HashSet::new());
                                        }
                                        content_case.get_mut(&cidx).unwrap().insert(c as u64);
                                    }
                                },
                                _ => unreachable!(),
                            },
                            MatchTableContent::WireCase(s, ranges) => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        println!("{:?}", header);
        println!("{:?}", content_case);
        // return MatchTable { target, segments };
    }
}

fn create_model(sheet: &Sheet) {
    let mut module = verilog_model::Module::new();
    let mut section: Section = Section::None;
    let mut row_iter = sheet.cells.iter().enumerate();
    while let Some((ridx, row)) = row_iter.next() {
        let mut col_iter = 0..row.len();
        if let Some(cidx) = col_iter.next() {
            match sheet.content(ridx, cidx) {
                Some((text, _offset)) => match text.as_str() {
                    "#input" => {
                        for cidx in col_iter {
                            if let Some((input, _)) = sheet.content(ridx, cidx) {
                                // println!("input: {}", &input);
                                let (_, signal) = parser::signal_def(&input).unwrap();
                                println!("[{}:{}]{}", signal.0 .0, signal.0 .1, signal.1);
                                module.new_input(signal.1.into(), signal.0 .0 + 1);
                            }
                        }
                    }

                    "#output" => {
                        for cidx in col_iter {
                            if let Some((output, _)) = sheet.content(ridx, cidx) {
                                println!("output: {}", &output);
                            }
                        }
                    }

                    "#match" => {
                        section = Section::Match(ridx);
                    }

                    "#end" => match section {
                        Section::Match(begin) => {
                            MatchTable::new(&mut module, sheet, begin, ridx);
                        }
                        Section::None => {
                            unreachable!()
                        }
                    },

                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn main() {
    let mut doc = excel::Excel::open("rv32_decode.xlsx");
    for s in doc.sheets() {
        let s = doc.sheet(&s);
        create_model(&s);
    }
}
