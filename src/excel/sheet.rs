use super::{CellPosition, CellRange};
use quick_xml::{self, events::Event, Reader};
use std::{cmp::Ordering, mem::replace, rc::Rc};

use super::get_xml_attribute;

#[derive(Debug)]
pub enum Cell {
    None,
    Primary {
        content: Rc<String>,
        size: CellPosition,
    },
    Merge {
        offset: CellPosition,
    },
}

#[allow(dead_code)]
impl Cell {
    pub fn merged_cell(&mut self, pos: CellPosition, range: &CellRange) {
        assert!(pos <= range.end);
        match pos.partial_cmp(&range.begin) {
            Some(Ordering::Equal) => match self {
                Cell::Primary { content: _, size } => {
                    *size = range.size();
                }
                _ => {
                    unreachable!()
                }
            },
            Some(Ordering::Greater) => {
                let _ = replace(
                    self,
                    Cell::Merge {
                        offset: CellPosition {
                            row: pos.row - range.begin.row,
                            col: pos.col - range.begin.col,
                        },
                    },
                );
            }
            _ => {
                unreachable!("{} ? {}", &pos, &range.begin);
            }
        }
    }
}

pub struct MergedCell {
    pub offset: CellPosition,
    pub size: CellPosition,
}

#[allow(dead_code)]
pub struct Sheet {
    pub cells: Vec<Vec<Cell>>,
}

#[allow(dead_code)]
impl Sheet {
    pub fn content(&self, ridx: usize, cidx: usize) -> Option<(Rc<String>, Option<MergedCell>)> {
        if ridx < self.cells.len() {
            let row = &self.cells[ridx];
            if cidx < row.len() {
                let cell = &row[cidx];
                match cell {
                    Cell::Primary { content, size } => {
                        return Some((
                            content.clone(),
                            Some(MergedCell {
                                offset: (0, 0).into(),
                                size: size.clone(),
                            }),
                        ));
                    }
                    Cell::Merge { offset } => {
                        match self
                            .content(ridx - (offset.row as usize), cidx - (offset.col as usize))
                        {
                            Some((content, Some(mut merged_cell))) => {
                                merged_cell.offset = offset.clone();
                                return Some((content, Some(merged_cell)));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    Cell::None => {
                        return None;
                    }
                }
            }
        }
        return None;
    }

    pub fn from_xml(xml: &str, shared_strings: &Vec<Rc<String>>) -> Sheet {
        let mut cells: Vec<Vec<Cell>> = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(64);

        let mut curr_row_cnt: Option<u16> = None;
        let mut curr_pos: Option<CellPosition> = None;
        let mut cell_value = false;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"row" => {
                        curr_row_cnt = get_xml_attribute(e, b"r").and_then(|a| unsafe {
                            std::str::from_utf8_unchecked(&a).parse::<u16>().ok()
                        });

                        if let Some(row) = curr_row_cnt {
                            for _ in cells.len()..(row as usize) {
                                cells.push(Vec::with_capacity(16));
                            }
                        } else {
                            unreachable!("invaild xml event of row in excel");
                        }
                    }
                    b"c" => {
                        curr_pos = get_xml_attribute(e, b"r").and_then(|a| unsafe {
                            Some(CellPosition::from(std::str::from_utf8_unchecked(&a)))
                        });

                        match &curr_pos {
                            Some(ref pos) => {
                                assert!(pos.row + 1 == curr_row_cnt.unwrap());
                                let current_row = &mut cells[(pos.row) as usize];
                                for _ in current_row.len()..(pos.col as usize + 1) {
                                    current_row.push(Cell::None);
                                }
                            }
                            None => {
                                unreachable!("invaild xml event col in excel");
                            }
                        }
                    }
                    b"v" => {
                        cell_value = true;
                    }

                    _ => {}
                },

                Ok(Event::Text(ref e)) => {
                    if cell_value == true {
                        let idx = unsafe {
                            let s = std::str::from_utf8_unchecked(&e);
                            s.parse::<usize>().expect(s)
                        };
                        match &curr_pos {
                            Some(ref pos) => {
                                // println!("{} -> {}", pos, shared_strings[idx]);
                                cells[pos.row as usize][pos.col as usize] = Cell::Primary {
                                    content: shared_strings[idx].clone(),
                                    size: CellPosition { row: 1, col: 1 },
                                };
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
                    b"row" => curr_row_cnt = None,
                    b"c" => curr_pos = None,
                    b"v" => cell_value = false,
                    _ => {}
                },

                Ok(Event::Empty(ref e)) => match e.name() {
                    b"c" => {
                        let current_row = &mut cells[curr_row_cnt.unwrap() as usize - 1];
                        current_row.push(Cell::None);
                    }
                    b"mergeCell" => {
                        let range = get_xml_attribute(e, b"ref")
                            .and_then(|a| unsafe {
                                let s = std::str::from_utf8_unchecked(&a);
                                Some(CellRange::from(s))
                            })
                            .unwrap();

                        for row in range.rows() {
                            for col in range.cols() {
                                let pos: CellPosition = (row, col).into();
                                cells[row as usize][col as usize].merged_cell(pos, &range);
                            }
                        }
                    }
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
