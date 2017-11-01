/// A cage in a KenKen puzzle. Every cell in a KenKen puzzle belongs to a cage.
/// Every cage has an operator and a target number.
#[derive(Debug, Deserialize, Serialize)]
pub struct Cage {

    /// The target number that must be produced using the numbers in this cage
    pub target: i32,

    /// The math operator that must be used with the numbers in the cage
    /// to produce the target number
    pub operator: Operator,

    /// A list of the positions of the cells in this cage
    pub cells: Vec<usize>,
}

/// The `Operator` enum represents each of the possible math operators
/// that can be in a cage.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum Operator { Add, Subtract, Multiply, Divide }

impl Operator {

    /// Retrieve the character representation of the symbol
    pub fn symbol(&self) -> char {
        match *self {
            Operator::Add      => '+',
            Operator::Subtract => '-',
            Operator::Multiply => '*',
            Operator::Divide   => '/',
        }
    }
}
