use self::CellVariable::{Solved, Unsolved};
use super::ValueSet;
use crate::puzzle::Value;

#[derive(Clone, Debug)]
pub(crate) enum CellVariable {
    Solved(Value),
    Unsolved(ValueSet),
}

impl CellVariable {
    pub fn unsolved_with_all(size: usize) -> CellVariable {
        Unsolved(ValueSet::with_all(size))
    }

    pub fn is_solved(&self) -> bool {
        matches!(*self, Solved(_))
    }

    pub fn is_unsolved(&self) -> bool {
        matches!(*self, Unsolved(_))
    }

    pub fn solved(&self) -> Option<i32> {
        match *self {
            Solved(value) => Some(value),
            _ => None,
        }
    }

    pub fn unsolved(&self) -> Option<&ValueSet> {
        match self {
            Unsolved(domain) => Some(domain),
            _ => None,
        }
    }

    pub fn unsolved_mut(&mut self) -> Option<&mut ValueSet> {
        match self {
            Unsolved(domain) => Some(domain),
            _ => None,
        }
    }

    pub fn unsolved_and_contains(&self, value: i32) -> bool {
        matches!(*self, Unsolved(ref domain) if domain.contains(value))
    }
}
