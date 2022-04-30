use super::signal::*;
use super::SignalWidth;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Shl, Shr, Sub};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LogicTree(Rc<LogicElem>);

#[allow(unused)]
#[derive(Debug)]
pub enum LogicElem {
    Unit(Wire),
    Combine(Vec<LogicTree>),
    Not(LogicTree),
    And(LogicTree, LogicTree),
    Or(LogicTree, LogicTree),
    Xor(LogicTree, LogicTree),
    TernaryCond(LogicTree, LogicTree, LogicTree),
    LogicShiftLeft(LogicTree, SignalWidth),
    LogicShiftRight(LogicTree, SignalWidth),
    Add(LogicTree, LogicTree),
    Sub(LogicTree, LogicTree),
    Mul(LogicTree, LogicTree),
    Div(LogicTree, LogicTree),
    Equal(LogicTree, LogicTree),
    NotEqual(LogicTree, LogicTree),
    GreateThan(LogicTree, LogicTree),
    GreateThanEqual(LogicTree, LogicTree),
    LessThan(LogicTree, LogicTree),
    LessThanEqual(LogicTree, LogicTree),
}

impl From<Wire> for LogicElem {
    fn from(w: Wire) -> LogicElem {
        return LogicElem::Unit(w);
    }
}

impl From<Wire> for LogicTree {
    fn from(w: Wire) -> LogicTree {
        return LogicTree(Rc::new(w.into()));
    }
}

impl From<LogicElem> for LogicTree {
    fn from(e: LogicElem) -> LogicTree {
        return LogicTree(e.into());
    }
}

#[allow(dead_code)]
impl LogicTree {
    pub fn ternary(self, true_branch: LogicTree, else_brance: LogicTree) -> LogicTree {
        return LogicElem::TernaryCond(self, true_branch, else_brance).into();
    }

    pub fn equal(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::Equal(self, rhs).into();
    }

    pub fn not_equal(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::NotEqual(self, rhs).into();
    }

    pub fn greate_than(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::GreateThan(self, rhs).into();
    }

    pub fn greate_than_equal(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::GreateThanEqual(self, rhs).into();
    }

    pub fn less_than(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::LessThan(self, rhs).into();
    }

    pub fn less_than_equal(self, rhs: LogicTree) -> LogicTree {
        return LogicElem::LessThanEqual(self, rhs).into();
    }
}

impl Not for LogicTree {
    type Output = Self;
    fn not(self) -> Self::Output {
        return LogicElem::Not(self).into();
    }
}

impl BitAnd for LogicTree {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        return LogicElem::And(self, rhs).into();
    }
}

impl BitOr for LogicTree {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        return LogicElem::Or(self, rhs).into();
    }
}

impl BitXor for LogicTree {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        return LogicElem::Xor(self, rhs).into();
    }
}

impl Shl<SignalWidth> for LogicTree {
    type Output = Self;
    fn shl(self, rhs: SignalWidth) -> Self::Output {
        return LogicElem::LogicShiftLeft(self, rhs).into();
    }
}

impl Shr<SignalWidth> for LogicTree {
    type Output = Self;
    fn shr(self, rhs: SignalWidth) -> Self::Output {
        return LogicElem::LogicShiftRight(self, rhs).into();
    }
}

impl Add for LogicTree {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        return LogicElem::Add(self, rhs).into();
    }
}

impl Sub for LogicTree {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        return LogicElem::Sub(self, rhs).into();
    }
}

impl Mul for LogicTree {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        return LogicElem::Mul(self, rhs).into();
    }
}

impl Div for LogicTree {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        return LogicElem::Div(self, rhs).into();
    }
}
