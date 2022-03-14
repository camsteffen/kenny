pub use self::operator::Operator;

use crate::error::InvalidPuzzle;
use crate::puzzle::CellId;

mod operator;

/// A cage in a KenKen puzzle
///
/// Every cell in a KenKen puzzle belongs to a cage.
/// Every cage has an operator and a target number.
#[derive(Debug, PartialEq)]
pub struct Cage {
    /// A list of the positions of the cells in this cage
    cell_ids: Box<[CellId]>,

    /// The math operator that must be used with the numbers in the cage
    /// to produce the target number
    operator: Operator,

    /// The target number that must be produced using the numbers in this cage
    target: i32,
}

impl Cage {
    pub fn new(
        cell_ids: impl Into<Box<[CellId]>>,
        operator: Operator,
        target: i32,
    ) -> Result<Self, InvalidPuzzle> {
        fn inner(
            mut cell_ids: Box<[CellId]>,
            operator: Operator,
            target: i32,
        ) -> Result<Cage, InvalidPuzzle> {
            cell_ids.sort_unstable();
            let cage = Cage {
                cell_ids,
                operator,
                target,
            };
            validate(&cage)?;
            Ok(cage)
        }
        inner(cell_ids.into(), operator, target)
    }

    /// The number on the cage
    pub fn target(&self) -> i32 {
        self.target
    }

    /// The math operator on the cage
    pub fn operator(&self) -> Operator {
        self.operator
    }

    /// The IDs of the cells in the cage
    pub fn cell_ids(&self) -> &[CellId] {
        &self.cell_ids
    }
}

fn validate(cage: &Cage) -> Result<(), InvalidPuzzle> {
    match (cage.operator, cage.cell_ids().len()) {
        (_, 0) => Err(InvalidPuzzle::new("cage cell_ids must not be empty".into())),
        (Operator::Nop, 1) => Ok(()),
        (Operator::Nop, _) => Err(InvalidPuzzle::new(
            "cage with multiple cells must have an operator".into(),
        )),
        (operator, 1) => Err(InvalidPuzzle::new(format!(
            "cage operator ({}) must have more than one cell",
            operator.symbol().unwrap()
        ))),
        _ => Ok(()),
    }
}
