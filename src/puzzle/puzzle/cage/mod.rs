mod operator;

pub use self::operator::Operator;

use crate::collections::square::SquareIndex;

// TODO rename to CageData and CageRef to Cage
/// A cage in a KenKen puzzle. Every cell in a KenKen puzzle belongs to a cage.
/// Every cage has an operator and a target number.
#[derive(Debug)]
pub struct Cage {

    /// The target number that must be produced using the numbers in this cage
    target: i32,

    /// The math operator that must be used with the numbers in the cage
    /// to produce the target number
    operator: Operator,

    /// A list of the positions of the cells in this cage
    pub(in super) cell_indices: Vec<SquareIndex>,
}

impl Cage {
    pub fn new(target: i32, operator: Operator, cell_indices: Vec<SquareIndex>) -> Self {
        Self { target, operator, cell_indices }
    }

    pub fn target(&self) -> i32 {
        self.target
    }

    pub fn operator(&self) -> Operator {
        self.operator
    }
}
