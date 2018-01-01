/// The `Operator` enum represents each of the possible math operators
/// that can be in a cage.
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Operator { Add, Subtract, Multiply, Divide, Nop }

impl Operator {

    /// Retrieve the character representation of the symbol
    pub fn symbol(&self) -> Option<char> {
        let symbol = match *self {
            Operator::Add      => '+',
            Operator::Subtract => '-',
            Operator::Multiply => '*',
            Operator::Divide   => '/',
            Operator::Nop      => return None,
        };
        Some(symbol)
    }

    /// Retrieve an `Operator` from its corresponding symbol
    pub fn from_symbol(c: char) -> Option<Operator> {
        let o = match c {
            '+' => Operator::Add,
            '-' => Operator::Subtract,
            '*' => Operator::Multiply,
            '/' => Operator::Divide,
            _ => return None
        };
        Some(o)
    }
}
