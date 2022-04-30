use std::ops::Shr;

use super::verilog_model::SignalWidth;

pub fn binary_format(width: SignalWidth, constant: u128) -> String {
    let width = width as usize;
    let mut result = String::with_capacity(width);
    for i in (0..width).rev() {
        result.push(if constant.shr(i) & 0x01 == 1 {
            '1'
        } else {
            '0'
        });
    }
    return result;
}
