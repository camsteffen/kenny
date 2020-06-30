//! A unary constraint is a simple constraint that applies to an individual puzzle cell without regard to the domain of
//! other cells.
//! For example, if a cage has two cells, a target of 5, and a plus operator, it is known that, for each cell in that
//! cage, the value must be less than 5.
//! These constraints may be applied to the puzzle markup one time at the beginning of the solving process.
//! They do not need to be re-checked as the solution progresses.

use std::cmp::Reverse;

use num::Integer;

use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::IsSquare;

use crate::puzzle::solve::markup::CellChanges;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::{CageRef, CellId, Operator};
use crate::puzzle::{Puzzle, Value};

/// Applies all unary constraints to cell domains. Returns a list of all affected cells by index.
pub(crate) fn apply_unary_constraints(puzzle: &Puzzle, changes: &mut CellChanges) {
    debug!("Reducing cell domains by cage-specific info");

    for cage in puzzle.cages() {
        reduce_cage(puzzle, cage, changes);
    }
}

fn reduce_cage(puzzle: &Puzzle, cage: CageRef<'_>, changes: &mut CellChanges) {
    match cage.operator() {
        Operator::Add => reduce_cage_add(puzzle, cage, changes),
        Operator::Multiply => reduce_cage_multiply(puzzle, cage, changes),
        Operator::Subtract => reduce_cage_subtract(puzzle, cage, changes),
        Operator::Divide => reduce_cage_divide(puzzle, cage, changes),
        Operator::Nop => {
            debug_assert_eq!(1, cage.cell_count());
            let cell = cage.cell(0);
            debug!("solving single cell cage at {:?}", cage.cell(0).coord());
            changes.solve(cell.id(), cage.target());
        }
    }
}

fn reduce_cage_add(puzzle: &Puzzle, cage: CageRef<'_>, changes: &mut CellChanges) {
    // if the cage has 2 cells and an even target,
    // the values cannot be half of the target
    if cage.cell_count() == 2 && cage.target().is_even() {
        let half = cage.target() / 2;
        for &cell in cage.cell_ids() {
            changes.remove_domain_value(cell, half);
        }
    }

    for &cell in cage.cell_ids() {
        let other_cells: Vec<CellId> = cage
            .cell_ids()
            .iter()
            .copied()
            .filter(|&i| i != cell)
            .collect_into(Vec::with_capacity(cage.cell_count() - 1));
        let (other_min, other_max) = cells_add_min_max(puzzle, &other_cells);
        let min = cage.target() - other_max;
        let max = cage.target() - other_min;
        let mut remove: Vec<Value> = Vec::new();
        if min > 1 {
            remove.extend(1..min);
        }
        if max < puzzle.width() as i32 {
            remove.extend((max + 1)..=puzzle.width() as i32);
        }
        for value in remove {
            changes.remove_domain_value(cell, value);
        }
    }
}

fn reduce_cage_multiply(puzzle: &Puzzle, cage: CageRef<'_>, changes: &mut CellChanges) {
    let target = cage.target();
    let non_factors: Vec<i32> = (2..=puzzle.width() as i32)
        .filter(|n| !target.is_multiple_of(n))
        .collect();
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
            changes.remove_domain_value(cell.id(), n);
        }
    }
}

fn reduce_cage_subtract(puzzle: &Puzzle, cage: CageRef<'_>, changes: &mut CellChanges) {
    if cage.target() <= puzzle.width() as i32 / 2 {
        return;
    }
    let start = puzzle.width() as i32 - cage.target() + 1;
    debug!(
        "values {}-{} cannot exist in cage at {:?}",
        start,
        cage.target(),
        cage.cell(0).coord()
    );
    for cell in cage.cells() {
        for n in start..=cage.target() {
            changes.remove_domain_value(cell.id(), n);
        }
    }
}

fn reduce_cage_divide(puzzle: &Puzzle, cage: CageRef<'_>, changes: &mut CellChanges) {
    let non_domain = {
        let mut non_domain = ValueSet::with_all(puzzle.width());
        for n in 1..=puzzle.width() as i32 / cage.target() {
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
            changes.remove_domain_value(cell.id(), n);
        }
    }
}

fn cells_add_min_max(puzzle: &Puzzle, cells: &[CellId]) -> (i32, i32) {
    if cells.len() == 1 {
        // simple case
        return (1, puzzle.width() as i32);
    }
    let group_sequence = cell_group_sizes(puzzle, &cells);
    group_sequence_min_max(&group_sequence, puzzle.width())
}

/// Splits cells into the smallest possible set of groups where each
/// group does not have any two cells on the same vector.
/// A list of group sizes in descending order is returned.
///
/// Example:
///
/// ```text
/// A B
/// B A C
/// ```
///
/// Returns: `[2, 2, 1]`
fn cell_group_sizes(puzzle: &Puzzle, cells: &[CellId]) -> Vec<usize> {
    let mut groups: Vec<Vec<CellId>> = Vec::with_capacity(cells.len());
    for &cell in cells {
        match groups.iter_mut().find(|group| {
            // find a group where none of the cells share a vector with this cell
            group
                .iter()
                .all(|&c| puzzle.shared_vector(c, cell).is_none())
        }) {
            // add the cell to the group
            Some(group) => group.push(cell),
            // otherwise start a new group
            None => groups.push(vec![cell]),
        };
    }

    let mut sizes: Vec<_> = groups.into_iter().map(|g| g.len()).collect();
    sizes.sort_unstable_by_key(|&e| Reverse(e));
    sizes
}

fn group_sequence_min_max(group_sequence: &[usize], puzzle_width: usize) -> (i32, i32) {
    group_sequence
        .iter()
        .enumerate()
        .map(|(i, &size)| (((i + 1) * size) as i32, ((puzzle_width - i) * size) as i32))
        .fold((0, 0), |(a, b), (c, d)| (a + c, b + d))
}

#[cfg(test)]
mod test {
    use crate::puzzle::solve::constraint::apply_unary_constraints;
    use crate::puzzle::solve::markup::{CellChanges, PuzzleMarkupChanges};
    use crate::puzzle::Puzzle;

    #[test]
    fn test() {
        let puzzle = Puzzle::parse(
            "4\n\
            AACC\n\
            AFFF\n\
            EEHG\n\
            EBBD\n\
            8* 5+ 4+ 1 8+ 9+ 4 2",
        )
        .unwrap();
        let mut changes = CellChanges::new();
        apply_unary_constraints(&puzzle, &mut changes);
        let mut expected = CellChanges::new();
        expected.remove_domain_value(0, 3);
        expected.remove_domain_value(1, 3);
        expected.remove_domain_value(2, 2);
        expected.remove_domain_value(2, 4);
        expected.remove_domain_value(3, 2);
        expected.remove_domain_value(3, 4);
        expected.remove_domain_value(4, 3);
        expected.remove_domain_value(5, 1);
        expected.remove_domain_value(6, 1);
        expected.remove_domain_value(7, 1);
        expected.solve(10, 2);
        expected.solve(11, 4);
        expected.solve(15, 1);
        assert_eq!(changes, expected);
    }
}
