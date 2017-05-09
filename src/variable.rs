use cell_domain::CellDomain;

#[derive(Clone)]
pub enum Variable {
    Solved(i32),
    Unsolved(CellDomain),
}

impl Variable {
    /*
    fn is_solved(&self) -> bool {
        match self {
            &Variable::Solved(_) => true,
            _ => false,
        }
    }
    */

    pub fn unsolved_with_all(size: usize) -> Variable {
        Variable::Unsolved(CellDomain::with_all(size))
    }

    pub fn is_solved(&self) -> bool {
        match self {
            &Variable::Solved(_) => true,
            _ => false,
        }
    }

    pub fn is_unsolved(&self) -> bool {
        match self {
            &Variable::Unsolved(_) => true,
            _ => false,
        }
    }

    pub fn solved(&self) -> Option<i32> {
        match self {
            &Variable::Solved(value) => Some(value),
            _ => None,
        }
    }

    pub fn unsolved(&self) -> Option<&CellDomain> {
        match self {
            &Variable::Unsolved(ref domain) => Some(domain),
            _ => None,
        }
    }

    pub fn unwrap_unsolved(&self) -> &CellDomain {
        match self {
            &Variable::Unsolved(ref d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    /*
    fn unwrap_unsolved_mut(&mut self) -> &mut CellDomain {
        match self {
            &mut Variable::Unsolved(ref mut d) => d,
            _ => panic!("Not Unsolved"),
        }
    }
    */
}

