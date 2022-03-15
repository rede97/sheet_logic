use super::CellPosition;
use quick_xml::{self, events::Event, Reader};
use std::rc::Rc;

use super::get_xml_attribute;

#[allow(dead_code)]
pub enum Cell {
    None,
    Primary(Rc<String>),
    Merge {
        primary: CellPosition,
        offset: CellPosition,
    },
}

#[allow(dead_code)]
pub struct Sheet {
    cells: Vec<Vec<Cell>>,
}

#[allow(dead_code)]
impl Sheet {
    pub fn from_xml(xml: &str, shared_strings: &Vec<Rc<String>>) -> Sheet {
        let mut cells: Vec<Vec<Cell>> = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(64);

        let mut row: Option<u16> = None;
        let mut pos: Option<CellPosition> = None;
        let mut value = false;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"row" => {
                        row = get_xml_attribute(e, b"r").and_then(|a| unsafe {
                            std::str::from_utf8_unchecked(&a).parse::<u16>().ok()
                        });

                        if let Some(row) = row {
                            for _ in cells.len()..(row as usize) {
                                cells.push(Vec::new());
                            }
                        } else {
                            unreachable!("invaild xml event of row in excel");
                        }
                    }
                    b"c" => {
                        pos = get_xml_attribute(e, b"r").and_then(|a| unsafe {
                            Some(CellPosition::from(std::str::from_utf8_unchecked(&a)))
                        });

                        match &pos {
                            Some(ref pos) => {
                                assert!(pos.row == row.unwrap());
                                let current_row = &mut cells[(pos.row - 1) as usize];
                                for _ in current_row.len()..(pos.col as usize) {
                                    current_row.push(Cell::None);
                                }
                            }
                            None => {
                                unreachable!("invaild xml event col in excel");
                            }
                        }
                    }
                    b"v" => {
                        value = true;
                    }

                    _ => {}
                },

                Ok(Event::Text(ref e)) => {
                    if value == true {
                        let idx = unsafe {
                            let s = std::str::from_utf8_unchecked(&e);
                            s.parse::<usize>().expect(s)
                        };
                        match &pos {
                            Some(ref pos) => {
                                println!("{} -> {}", pos, shared_strings[idx]);
                            }
                            None => {
                                unreachable!("ref {}", std::str::from_utf8(&e).unwrap());
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }

                Ok(Event::End(ref e)) => match e.name() {
                    b"row" => row = None,
                    b"c" => pos = None,
                    b"v" => value = false,
                    _ => {}
                },

                Ok(Event::Eof) => {
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    panic!("{} {:?}", reader.buffer_position(), e);
                }
            }
        }

        return Sheet { cells };
    }
}
