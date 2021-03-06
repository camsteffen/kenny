/// The math operators found in each cage of a puzzle
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Nop,
}

impl Operator {
    /// Retrieve the unicode character representation of the symbol
    pub fn display_symbol(self) -> Option<char> {
        let symbol = match self {
            Operator::Add => '+',
            Operator::Subtract => '−',
            Operator::Multiply => '×',
            Operator::Divide => '÷',
            Operator::Nop => return None,
        };
        Some(symbol)
    }

    /// Retrieve the character representation of the symbol
    pub fn symbol(self) -> Option<char> {
        let symbol = match self {
            Operator::Add => '+',
            Operator::Subtract => '-',
            Operator::Multiply => '*',
            Operator::Divide => '/',
            Operator::Nop => return None,
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
            _ => return None,
        };
        Some(o)
    }
}
