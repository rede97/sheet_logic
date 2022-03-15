mod excel;
mod verilog_model;
fn main() {
    let mut doc = excel::Excel::open("rv32_decode.xlsx");
    println!("{:?}", doc.sheets());
    for s in doc.sheets() {
        doc.sheet(&s);
    }
}
