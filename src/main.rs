mod excel;
mod verilog_model;
use excel::{Cell, CellPosition, Sheet};
use std::rc::Rc;

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

fn create_model(sheet: &Sheet) {
    let module = verilog_model::Module::new();
    for (ridx, row) in sheet.cells.iter().enumerate() {
        for (cidx, cell) in row.iter().enumerate() {
            match cell_content(cell, sheet, (ridx, cidx)) {
                Some((text, offset)) => {
                    
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let mut doc = excel::Excel::open("rv32_decode.xlsx");
    for s in doc.sheets() {
        let _ = doc.sheet(&s);
    }
}
