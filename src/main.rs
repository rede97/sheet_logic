mod excel;
mod parser;
#[allow(dead_code)]
mod verilog_model;
use excel::{Cell, CellPosition, Sheet};
use std::{rc::Rc, collections::HashSet};
use verilog_model::{Module, Signal, SignalKey, SignalSource, Wire};

enum Section {
    None,
    Match(usize),
}

fn cell_content(
    cell: &Cell,
    sheet: &Sheet,
    idx: (usize, usize),
) -> Option<(Rc<String>, Option<CellPosition>)> {
    return match cell {
        Cell::None => None,
        Cell::Primary(s) => Some((s.clone(), None)),
        Cell::Merge { offset } => {
            if let Cell::Primary(s) =
                &sheet.cells[idx.0 - (offset.row as usize)][idx.1 - (offset.col as usize)]
            {
                Some((s.clone(), Some(offset.clone())))
            } else {
                unreachable!("invaild merged cell");
            }
        }
    };
}

struct MatchTable {
    target: Wire,
    segments: Vec<(SignalKey, HashSet<u64>)>,
}

impl MatchTable {
    pub fn new(model: &mut Module, sheet: &Sheet, begin: usize, end: usize) -> Self {
        let row = &mut sheet.cells.iter().skip(begin).take(end - begin).enumerate();
        let (ridx, header) = row.next().unwrap();
        let target_signal_str = cell_content(&header[1], sheet, (ridx, 1))
            .and_then(|(c, _)| Some(c))
            .unwrap();
        let (_, ((h, l), signal_name)) =
            parser::signal(&target_signal_str).expect(&target_signal_str);
        let target_signal = model.get_signals().get(signal_name).unwrap().clone();
        let target = target_signal.range(h, l).unwrap();

        let mut segments: Vec<(SignalKey, HashSet<u64>)> = Vec::new();
        let (ridx, segs_row) = row.next().unwrap();
        for (cidx, seg) in segs_row.iter().enumerate() {
            let mdl = model.get_signals_mut();
            match cell_content(seg, sheet, (ridx, cidx)) {
                Some((c, None)) => {
                    let (_, ((h, l), alias)) = parser::range_alias(&c).unwrap();
                    let seg_key: SignalKey = alias
                        .unwrap_or(format!("{}_{}_{}", signal_name, h, l))
                        .into();
                    let seg_src = SignalSource::Wire(target_signal.range(h, l).unwrap());
                    let seg_signal = Signal::new(seg_key.clone(), h - l + 1, seg_src);
                    mdl.insert(seg_key.clone(), seg_signal);
                    segments.push((seg_key.clone(), HashSet::new()));
                }
                _ => {}
            }
        }
        println!("{:?}", segments);
        return MatchTable { target, segments };
    }
}

fn create_model(sheet: &Sheet) {
    let mut module = verilog_model::Module::new();
    let mut section: Section = Section::None;
    let mut row_iter = sheet.cells.iter().enumerate();
    while let Some((ridx, row)) = row_iter.next() {
        let mut col_iter = row.iter().enumerate();
        if let Some((cidx, cell)) = col_iter.next() {
            match cell_content(cell, sheet, (ridx, cidx)) {
                Some((text, offset)) => match text.as_str() {
                    "#input" => {
                        for (cidx, cell) in col_iter {
                            if let Some((input, _)) = cell_content(cell, sheet, (ridx, cidx)) {
                                // println!("input: {}", &input);
                                let (_, signal) = parser::signal(&input).unwrap();
                                println!("[{}:{}]{}", signal.0 .0, signal.0 .1, signal.1);
                                module.new_input(signal.1.into(), signal.0 .0 + 1);
                            }
                        }
                    }

                    "#output" => {
                        for (cidx, cell) in col_iter {
                            if let Some((output, _)) = cell_content(cell, sheet, (ridx, cidx)) {
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
                        _ => {}
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
