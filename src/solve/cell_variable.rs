use super::CellDomain;

#[derive(Clone)]
pub enum CellVariable {
    Solved(i32),
    Unsolved(CellDomain),
}

impl CellVariable {
    /*
    fn is_solved(&self) -> bool {
        match self {
            &Variable::Solved(_) => true,
            _ => false,
        }
    }
    */

    pub fn unsolved_with_all(size: usize) -> CellVariable {
        CellVariable::Unsolved(CellDomain::with_all(size))
    }

    pub fn is_solved(&self) -> bool {
        match *self {
            CellVariable::Solved(_) => true,
            _ => false,
        }
    }

    pub fn is_unsolved(&self) -> bool {
        match *self {
            CellVariable::Unsolved(_) => true,
            _ => false,
        }
    }

    pub fn solved(&self) -> Option<i32> {
        match *self {
            CellVariable::Solved(value) => Some(value),
            _ => None,
        }
    }

    pub fn unsolved(&self) -> Option<&CellDomain> {
        match *self {
            CellVariable::Unsolved(ref domain) => Some(domain),
            _ => None,
        }
    }

    pub fn unwrap_solved(&self) -> i32 {
        match *self {
            CellVariable::Solved(val) => val,
            _ => panic!("Not Solved"),
        }
    }

    pub fn unwrap_unsolved(&self) -> &CellDomain {
        match *self {
            CellVariable::Unsolved(ref d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    /*
    pub fn unwrap_unsolved_mut(&mut self) -> &mut RangeDomain {
        match self {
            &mut Variable::Unsolved(ref mut d) => d,
            _ => panic!("Not Unsolved"),
        }
    }
    */
}

