use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{hex_digit1, u128 as uint128, u16 as uint16},
    combinator::map_res,
    sequence::{pair, tuple},
    IResult,
};

fn from_hex(input: &str) -> Result<u128, std::num::ParseIntError> {
    u128::from_str_radix(input, 16)
}

fn from_bin(input: &str) -> Result<u128, std::num::ParseIntError> {
    u128::from_str_radix(input, 2)
}

fn is_bin_digit(c: char) -> bool {
    c.is_digit(2)
}

// fn dec_primary(input: &str) -> IResult<&str, u128> {
//     map_res(take_while1(is_bin_digit), from_dec)(input)
// }

fn constant_hex(input: &str) -> IResult<&str, u128> {
    let (s, (_, num)) = pair(tag("'h"), map_res(hex_digit1, from_hex))(input)?;
    return Ok((s, num));
}

fn constant_dec(input: &str) -> IResult<&str, u128> {
    let (s, (_, num)) = pair(tag("'d"), uint128)(input)?;
    return Ok((s, num));
}

fn constant_bin(input: &str) -> IResult<&str, u128> {
    let (s, (_, num)) = pair(tag("'b"), map_res(take_while1(is_bin_digit), from_bin))(input)?;
    return Ok((s, num));
}

fn bit_width(mut num: u128) -> u16 {
    let mut w = 0;
    while num > 0 {
        num = num >> 1;
        w = w + 1;
    }
    w
}

fn constant(input: &str) -> IResult<&str, (u16, u128)> {
    let (s, (w, n)) = pair(uint16, alt((constant_hex, constant_dec, constant_bin)))(input)?;
    assert!(w >= bit_width(n));
    Ok((s, (w, n)))
}

#[cfg(test)]
mod tests {
    use super::constant;

    #[test]
    fn constant_hex_test() {
        let (_, r) = constant("16'h3a4b").unwrap();
        assert_eq!((16, 0x3a4b), r);
        let (_, r) = constant("16'd2344").unwrap();
        assert_eq!((16, 2344), r);
        let (_, r) = constant("32'hfabcd444").unwrap();
        assert_eq!((32, 0xfabc_d444), r);
        let (_, r) = constant("8'b11100000").unwrap();
        assert_eq!((8, 0b11100000), r);
    }
}
