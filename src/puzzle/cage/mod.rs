mod operator;

pub use self::operator::Operator;

use collections::square::SquareIndex;

/// A cage in a KenKen puzzle. Every cell in a KenKen puzzle belongs to a cage.
/// Every cage has an operator and a target number.
#[derive(Debug)]
pub struct Cage {

    /// The target number that must be produced using the numbers in this cage
    pub target: i32,

    /// The math operator that must be used with the numbers in the cage
    /// to produce the target number
    pub operator: Operator,

    /// A list of the positions of the cells in this cage
    /// TODO rename to cell_indices
    pub cells: Vec<SquareIndex>,
}
