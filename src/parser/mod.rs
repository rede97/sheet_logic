#[allow(dead_code)]
mod constant;
#[allow(dead_code)]
mod signal;
#[allow(dead_code)]
mod match_cmd;

pub use constant::*;
pub use signal::*;
pub use match_cmd::*;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::recognize,
    multi::many0_count,
    sequence::pair,
    IResult,
};

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}
