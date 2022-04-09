use super::identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u16 as uint16,
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    IResult,
};

pub fn signal_range(input: &str) -> IResult<&str, (u16, u16)> {
    return delimited(tag("["), separated_pair(uint16, tag(":"), uint16), tag("]"))(input);
}

/// like: [31:0]signal
pub fn signal_def(input: &str) -> IResult<&str, ((u16, u16), &str)> {
    return pair(signal_range, identifier)(input);
}

/// like: [31:0], [31:0]alias
pub fn range_alias(input: &str) -> IResult<&str, ((u16, u16), Option<String>)> {
    let (input, ((h, l), alias)) = pair(signal_range, opt(identifier))(input)?;
    let alias = alias.and_then(|s| Some(String::from(s)));
    return Ok((input, ((h, l), alias)));
}

fn signal_ref_ringle_range(input: &str) -> IResult<&str, (u16, u16)> {
    let (input, idx) = uint16(input)?;
    return Ok((input, (idx, idx)));
}
/// like [20|31:24]
pub fn signal_ref_range(input: &str) -> IResult<&str, Vec<(u16, u16)>> {
    return delimited(
        tag("["),
        separated_list1(
            tag("|"),
            alt((
                separated_pair(uint16, tag(":"), uint16),
                signal_ref_ringle_range,
            )),
        ),
        tag("]"),
    )(input);
}

pub fn sginal_ref(input: &str) -> IResult<&str, (&str, Option<Vec<(u16, u16)>>)> {
    return pair(identifier, opt(signal_ref_range))(input);
}
