use super::identifier;
use super::{constant, sginal_ref};
use crate::{MatchTableColumn, MatchTableContent};
use nom::{branch::alt, bytes::complete::tag, sequence::delimited, IResult};

fn match_flag(input: &str) -> IResult<&str, MatchTableColumn> {
    let (input, prefix) = delimited(tag("#flag("), identifier, tag(")"))(input)?;
    return Ok((input, MatchTableColumn::Flag(prefix.into())));
}

fn match_primary(input: &str) -> IResult<&str, MatchTableColumn> {
    let (input, prefix) = delimited(tag("#primary("), identifier, tag(")"))(input)?;
    return Ok((input, MatchTableColumn::Primary(prefix.into())));
}

pub fn match_cmd(input: &str) -> IResult<&str, MatchTableColumn> {
    return alt((match_flag, match_primary))(input);
}

fn match_wire_case(input: &str) -> IResult<&str, MatchTableContent> {
    let (input, r) = sginal_ref(input)?;
    return Ok((input, MatchTableContent::Signal(r.0, r.1)));
}

fn match_constant(input: &str) -> IResult<&str, MatchTableContent> {
    let (input, c) = constant(input)?;
    return Ok((input, MatchTableContent::Constant(c.0, c.1)));
}

pub fn match_content(input: &str) -> IResult<&str, MatchTableContent> {
    return alt((match_wire_case, match_constant))(input);
}
