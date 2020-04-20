use crate::collections::square::SquareIndex;
use num::Integer;
use crate::puzzle::{Operator, CageRef};
use crate::puzzle::Puzzle;
use crate::puzzle::solve::CellDomain;
use std::collections::BTreeSet;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;

/// Applies all unary constraints to cell domains. Returns a list of all affected cells by index.
pub fn apply_unary_constraints(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) -> Vec<SquareIndex> {
    let affected_cells = BTreeSet::new();
    reduce_domains_by_cage(puzzle, change);
    affected_cells.into_iter().collect()
}

fn reduce_domains_by_cage(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) {
    debug!("reducing cell domains by cage-specific info");

    for cage in puzzle.cages().iter() {
        reduce_cage(puzzle.width(), cage, change);
    }
}

fn reduce_cage(puzzle_width: usize, cage: CageRef, change: &mut PuzzleMarkupChanges) {
    match cage.operator() {
        Operator::Add => {
            let start = cage.target() - cage.cell_count() as i32 + 2;
            if start > puzzle_width as i32 { return }
            debug!("values {}-{} cannot exist in cage at {:?}", start, puzzle_width,
                cage.cell(0).coord());
            for cell in cage.cells() {
                for n in start..=puzzle_width as i32 {
                    change.remove_value_from_cell(cell.index(), n);
                }
            }
        },
        Operator::Multiply => {
            let non_factors = (2..=puzzle_width as i32)
                .filter(|n| !cage.target().is_multiple_of(n))
                .collect::<Vec<_>>();
            if non_factors.is_empty() { return }
            debug!("values {:?} cannot exist in cage at {:?}", non_factors,
                cage.cell(0).coord());
            for cell in cage.cells() {
                for &n in &non_factors {
                    change.remove_value_from_cell(cell.index(), n);
                }
            }
        },
        Operator::Subtract => {
            let size = puzzle_width as i32;
            if cage.target() <= size / 2 { return }
            let start = size - cage.target() + 1;
            debug!("values {}-{} cannot exist in cage at {:?}", start, cage.target(),
                cage.cell(0).coord());
            for cell in cage.cells() {
                for n in start..=cage.target() {
                    change.remove_value_from_cell(cell.index(), n);
                }
            }
        },
        Operator::Divide => {
            let mut non_domain = CellDomain::with_all(puzzle_width);
            for n in 1..=puzzle_width as i32 / cage.target() {
                non_domain.remove(n);
                non_domain.remove(n * cage.target());
            }
            if non_domain.is_empty() { return }
            debug!("values {:?} cannot exist in cage at {:?}", non_domain.iter().collect::<Vec<_>>(),
                cage.cell(0).coord());
            for cell in cage.cells() {
                for n in &non_domain {
                    change.remove_value_from_cell(cell.index(), n);
                }
            }
        },
        Operator::Nop => {
            debug_assert_eq!(1, cage.cell_count());
            let cell = cage.cell(0);
            debug!("solving single cell cage at {:?}", cage.cell(0).coord());
            change.solve_cell(cell.index(), cage.target());
        }
    }
}
