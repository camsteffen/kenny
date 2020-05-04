use self::CellVariable::{Solved, Unsolved};
use super::ValueSet;
use crate::puzzle::Value;

#[derive(Clone)]
pub enum CellVariable {
    Solved(Value),
    Unsolved(ValueSet),
}

impl CellVariable {
    pub fn unsolved_with_all(size: usize) -> CellVariable {
        Unsolved(ValueSet::with_all(size))
    }

    pub fn is_solved(&self) -> bool {
        match *self {
            Solved(_) => true,
            _ => false,
        }
    }

    pub fn is_unsolved(&self) -> bool {
        match *self {
            Unsolved(_) => true,
            _ => false,
        }
    }

    pub fn solve(&mut self) -> Option<i32> {
        let solution = self.unwrap_unsolved().single_value();
        if let Some(solution) = solution {
            *self = Solved(solution)
        }
        solution
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

    pub fn unwrap_solved(&self) -> i32 {
        match self {
            Solved(val) => *val,
            _ => panic!("Not Solved"),
        }
    }

    pub fn unwrap_unsolved(&self) -> &ValueSet {
        match self {
            Unsolved(d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    pub fn unwrap_unsolved_mut(&mut self) -> &mut ValueSet {
        match self {
            Unsolved(d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    pub fn remove_from_domain(&mut self, value: i32) -> bool {
        self.unwrap_unsolved_mut().remove(value)
    }

    pub fn unsolved_and_contains(&self, value: i32) -> bool {
        match self {
            Solved(_) => false,
            Unsolved(ref domain) => domain.contains(value),
        }
    }
}
