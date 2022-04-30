mod excel;
#[allow(dead_code)]
mod match_table;
mod parser;
#[allow(dead_code)]
mod verilog_model;
use excel::Sheet;
use match_table::*;
mod utils;

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
                                println!("input [{}:{}]{}", signal.0 .0, signal.0 .1, signal.1);
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
                            MatchTable::parse(&mut module, sheet, begin, ridx);
                        }
                        Section::None => {
                            unreachable!()
                        }
                    },

                    "#wire" => {
                        for cidx in col_iter {
                            if let Some((input, _)) = sheet.content(ridx, cidx) {
                                let (_, signal) = parser::signal_def(&input).unwrap();
                                println!("wire [{}:{}]{}", signal.0 .0, signal.0 .1, signal.1);
                                module.new_signal(signal.1.into(), signal.0 .0 + 1);
                            }
                        }
                    }

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
