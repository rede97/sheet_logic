use quick_xml::{self, events::Event, Reader};
use std::{fs::File, io::Read};
use zip;

mod excel;
mod verilog_model;
fn main() {
    let mut doc = excel::Excel::open("rv32_decode.xlsx");
    println!("{:?}", doc.sheets());
}
