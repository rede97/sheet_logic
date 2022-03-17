use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Display};
use std::ops::Range;

mod cell_position_parser {
    use super::{CellPosition, CellRange};
    use nom::{
        bytes::complete::tag,
        character::complete::{alpha1, u16 as uint16},
        sequence::{pair, tuple},
        IResult,
    };

    fn parse_raw_tuple(t: (&str, u16)) -> CellPosition {
        let col = t.0.chars().fold(0, |sum, c| {
            (c.to_ascii_uppercase() as u16) - ('A' as u16) + sum
        });
        return CellPosition { row: t.1 - 1, col };
    }

    fn cell_position(input: &str) -> IResult<&str, (&str, u16)> {
        pair(alpha1, uint16)(input)
    }

    pub fn cell(pos: &str) -> CellPosition {
        let (_, t) = cell_position(pos).expect(pos);
        return parse_raw_tuple(t);
    }

    pub fn range(range: &str) -> CellRange {
        let (_, (begin, _, end)) =
            tuple((cell_position, tag(":"), cell_position))(range).expect(range);
        return CellRange {
            begin: parse_raw_tuple(begin),
            end: parse_raw_tuple(end),
        };
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn excel_position() {
            assert_eq!(CellPosition { row: 7, col: 2 }, cell("C8"));
            assert_eq!(
                CellRange {
                    begin: "C8".into(),
                    end: "C9".into()
                },
                range("C8:C9")
            );
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CellPosition {
    pub row: u16,
    pub col: u16,
}

#[allow(dead_code)]
impl CellPosition {
    #[inline]
    pub fn new(pos: &str) -> CellPosition {
        return cell_position_parser::cell(pos);
    }

    /// (row, col)
    pub fn from_tuple(t: (u16, u16)) -> CellPosition {
        return CellPosition { row: t.0, col: t.1 };
    }

    /// (row, col)
    pub fn tuple(&self) -> (u16, u16) {
        return (self.row, self.col);
    }
}

impl From<&str> for CellPosition {
    #[inline]
    fn from(pos: &str) -> Self {
        return CellPosition::new(pos);
    }
}

impl From<(u16, u16)> for CellPosition {
    #[inline]
    fn from(t: (u16, u16)) -> Self {
        return CellPosition::from_tuple(t);
    }
}

impl PartialOrd for CellPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return match (self.row.cmp(&other.row), self.col.cmp(&other.col)) {
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            (Ordering::Greater | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
                Some(Ordering::Greater)
            }
            (Ordering::Less | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
                Some(Ordering::Less)
            }
            _ => None,
        };
    }
}

impl Display for CellPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

#[derive(Debug, PartialEq)]
pub struct CellRange {
    pub begin: CellPosition,
    pub end: CellPosition,
}

#[allow(dead_code)]
impl CellRange {
    #[inline]
    fn new(range: &str) -> CellRange {
        return cell_position_parser::range(range);
    }

    pub fn rows(&self) -> Range<u16> {
        (self.begin.row)..(self.end.row + 1)
    }

    pub fn cols(&self) -> Range<u16> {
        (self.begin.col)..(self.end.col + 1)
    }
}

impl From<&str> for CellRange {
    #[inline]
    fn from(range: &str) -> Self {
        return CellRange::new(range);
    }
}

impl Display for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.begin, self.end)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cmp::{Ordering, PartialOrd};

    #[test]
    fn test_cmp() {
        assert_eq!(
            CellPosition::new("B6").partial_cmp(&("B6".into())),
            Some(Ordering::Equal)
        );
        assert_eq!(
            CellPosition::new("B6").partial_cmp(&("B5".into())),
            Some(Ordering::Greater)
        );
        assert_eq!(
            CellPosition::new("B6").partial_cmp(&("B7".into())),
            Some(Ordering::Less)
        );
        assert_eq!(CellPosition::new("B6").partial_cmp(&("A7".into())), None);
    }
}
