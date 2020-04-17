use crate::collections::square::SquareIndex;
use num::Integer;
use crate::puzzle::Operator;
use crate::puzzle::Puzzle;
use crate::puzzle::solve::CellDomain;
use std::collections::BTreeSet;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::Cage;

/// Applies all unary constraints to cell domains. Returns a list of all affected cells by index.
pub fn apply_unary_constraints(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) -> Vec<SquareIndex> {
    let affected_cells = BTreeSet::new();
    reduce_domains_by_cage(puzzle, change);
    affected_cells.into_iter().collect()
}

fn reduce_domains_by_cage(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) {
    debug!("reducing cell domains by cage-specific info");

    for cage in &puzzle.cages {
        reduce_cage(puzzle.width, cage, change);
    }
}

fn reduce_cage(puzzle_width: u32, cage: &Cage, change: &mut PuzzleMarkupChanges) {
    match cage.operator {
        Operator::Add => {
            let start = cage.target - cage.cells.len() as i32 + 2;
            if start > puzzle_width as i32 { return }
            debug!("values {}-{} cannot exist in cage at {:?}", start, puzzle_width,
                cage.cells[0].as_coord(puzzle_width as usize));
            for &pos in &cage.cells {
                for n in start..=puzzle_width as i32 {
                    change.remove_value_from_cell(pos, n);
                }
            }
        },
        Operator::Multiply => {
            let non_factors = (2..=puzzle_width as i32)
                .filter(|n| !cage.target.is_multiple_of(n))
                .collect::<Vec<_>>();
            if non_factors.is_empty() { return }
            debug!("values {:?} cannot exist in cage at {:?}", non_factors,
                cage.cells[0].as_coord(puzzle_width as usize));
            for &pos in &cage.cells {
                for &n in &non_factors {
                    change.remove_value_from_cell(pos, n);
                }
            }
        },
        Operator::Subtract => {
            let size = puzzle_width as i32;
            if cage.target <= size / 2 { return }
            let start = size - cage.target + 1;
            debug!("values {}-{} cannot exist in cage at {:?}", start, cage.target,
                cage.cells[0].as_coord(puzzle_width as usize));
            for &pos in &cage.cells {
                for n in start..=cage.target {
                    change.remove_value_from_cell(pos, n);
                }
            }
        },
        Operator::Divide => {
            let mut non_domain = CellDomain::with_all(puzzle_width);
            for n in 1..=puzzle_width as i32 / cage.target {
                non_domain.remove(n);
                non_domain.remove(n * cage.target);
            }
            if non_domain.is_empty() { return }
            debug!("values {:?} cannot exist in cage at {:?}", non_domain.iter().collect::<Vec<_>>(),
                cage.cells[0].as_coord(puzzle_width as usize));
            for &pos in &cage.cells {
                for n in &non_domain {
                    change.remove_value_from_cell(pos, n);
                }
            }
        },
        Operator::Nop => {
            debug_assert_eq!(1, cage.cells.len());
            let pos = cage.cells[0];
            debug!("solving single cell cage at {:?}", cage.cells[0].as_coord(puzzle_width as usize));
            change.solve_cell(pos, cage.target);
        }
    }
}
