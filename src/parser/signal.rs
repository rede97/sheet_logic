use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, u16 as uint16},
    combinator::opt,
    combinator::recognize,
    multi::many0_count,
    sequence::pair,
    sequence::tuple,
    IResult,
};

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn signal_range(input: &str) -> IResult<&str, (u16, u16)> {
    let (input, (_, h, _, l, _)) = tuple((tag("["), uint16, tag(":"), uint16, tag("]")))(input)?;
    return Ok((input, (h, l)));
}

pub fn signal(input: &str) -> IResult<&str, ((u16, u16), &str)> {
    return tuple((signal_range, identifier))(input);
}

pub fn range_alias(input: &str) -> IResult<&str, ((u16, u16), Option<String>)> {
    let (input, ((h, l), alias)) = tuple((signal_range, opt(identifier)))(input)?;
    let alias = alias.and_then(|s| Some(String::from(s)));
    return Ok((input, ((h, l), alias)));
}
