//! A unary constraint is a simple constraint that applies to an individual puzzle cell without regard to the domain of
//! other cells.
//! For example, if a cage has two cells, a target of 5, and a plus operator, it is known that, for each cell in that
//! cage, the value must be less than 5.
//! These constraints may be applied to the puzzle markup one time at the beginning of the solving process.
//! They do not need to be re-checked as the solution progresses.

use crate::collections::square::Vector;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::Puzzle;
use crate::puzzle::{CageRef, CellId, Operator};
use ahash::AHashSet;
use num::Integer;
use std::collections::BTreeSet;

/// Applies all unary constraints to cell domains. Returns a list of all affected cells by index.
pub fn apply_unary_constraints(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) -> Vec<CellId> {
    let affected_cells = BTreeSet::new();
    reduce_domains_by_cage(puzzle, change);
    affected_cells.into_iter().collect()
}

fn reduce_domains_by_cage(puzzle: &Puzzle, change: &mut PuzzleMarkupChanges) {
    debug!("reducing cell domains by cage-specific info");

    for cage in puzzle.cages() {
        reduce_cage(puzzle, cage, change);
    }
}

fn reduce_cage(puzzle: &Puzzle, cage: CageRef<'_>, change: &mut PuzzleMarkupChanges) {
    let puzzle_width = puzzle.width();
    match cage.operator() {
        Operator::Add => {
            let largest = largest_value_add(cage);
            if largest < puzzle_width as i32 {
                debug!(
                    "values greater than {} cannot exist in cage at {:?}",
                    largest,
                    cage.coord()
                );
                for cell in cage.cells() {
                    for n in (largest + 1)..=puzzle_width as i32 {
                        change.remove_value_from_cell(cell.id(), n);
                    }
                }
            }
        }
        Operator::Multiply => {
            let non_factors = (2..=puzzle_width as i32)
                .filter(|n| !cage.target().is_multiple_of(n))
                .collect::<Vec<_>>();
            if non_factors.is_empty() {
                return;
            }
            debug!(
                "values {:?} cannot exist in cage at {:?}",
                non_factors,
                cage.cell(0).coord()
            );
            for cell in cage.cells() {
                for &n in &non_factors {
                    change.remove_value_from_cell(cell.id(), n);
                }
            }
        }
        Operator::Subtract => {
            let size = puzzle_width as i32;
            if cage.target() <= size / 2 {
                return;
            }
            let start = size - cage.target() + 1;
            debug!(
                "values {}-{} cannot exist in cage at {:?}",
                start,
                cage.target(),
                cage.cell(0).coord()
            );
            for cell in cage.cells() {
                for n in start..=cage.target() {
                    change.remove_value_from_cell(cell.id(), n);
                }
            }
        }
        Operator::Divide => {
            let non_domain = {
                let mut non_domain = ValueSet::with_all(puzzle_width);
                for n in 1..=puzzle_width as i32 / cage.target() {
                    non_domain.remove(n);
                    non_domain.remove(n * cage.target());
                }
                non_domain
            };
            if non_domain.is_empty() {
                return;
            }
            debug!(
                "values {:?} cannot exist in cage at {:?}",
                non_domain.iter().collect::<Vec<_>>(),
                cage.cell(0).coord()
            );
            for cell in cage.cells() {
                for n in &non_domain {
                    change.remove_value_from_cell(cell.id(), n);
                }
            }
        }
        Operator::Nop => {
            debug_assert_eq!(1, cage.cell_count());
            let cell = cage.cell(0);
            debug!("solving single cell cage at {:?}", cage.cell(0).coord());
            change.solve_cell(cell.id(), cage.target());
        }
    }
}

/// Calculates the largest possible value in an addition cage
fn largest_value_add(cage: CageRef<'_>) -> i32 {
    // simple case
    if cage.cell_count() == 2 {
        return cage.target() - 1;
    }

    #[derive(Default)]
    struct Group {
        cells: Vec<CellId>,
        vectors: AHashSet<Vector>,
    }

    let mut groups: Vec<Group> = Vec::with_capacity(cage.cell_count());

    // split cells into groups where each group does not have any cells that share a vector
    for cell in cage.cells() {
        let group = groups.iter_mut().find(|group| {
            // find a group where none of the cells share a vector with this cell
            cell.vectors().iter().all(|v| !group.vectors.contains(v))
        });
        let group = match group {
            Some(group) => group,
            None => {
                // otherwise start a new group
                groups.push(Group::default());
                groups.last_mut().unwrap()
            }
        };
        // add this cell and its vectors to the group
        group.cells.push(cell.id());
        group.vectors.extend(cell.vectors().iter());
    }

    // sort bigger groups first
    groups.sort_unstable_by(|a, b| b.cells.len().cmp(&a.cells.len()));

    // fill in the cage with the smallest possible sum
    let smallest_sum = groups
        .iter()
        .enumerate()
        // fill in cells of the first group with 1, the second group with 2, etc. and sum the values
        .map(|(i, group)| (i + 1) * group.cells.len())
        // sum the groups
        .sum::<usize>();

    // subtract one of the cells from the group with the largest values
    let smallest_sum_with_blank = (smallest_sum - groups.len()) as i32;

    // TODO largest value PER GROUP
    // largest possible cell value
    cage.target() - smallest_sum_with_blank
}
