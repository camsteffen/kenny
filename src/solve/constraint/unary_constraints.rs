use collections::Square;
use collections::square::SquareIndex;
use num::Integer;
use puzzle::Operator;
use puzzle::Puzzle;
use solve::CellDomain;
use std::collections::BTreeSet;

/// Applies all unary constraints to cell domains. Returns a list of all affected cells by index.
pub fn apply_unary_constraints(puzzle: &Puzzle, cell_domains: &mut Square<CellDomain>) -> Vec<SquareIndex> {
    let affected_cells = BTreeSet::new();
    reduce_domains_by_cage(puzzle, cell_domains, &mut affected_cells);
    affected_cells.into_iter().collect()
}

fn reduce_domains_by_cage(puzzle: &Puzzle, cell_domains: &mut Square<CellDomain>, affected_cells: &mut BTreeSet<SquareIndex>) {
    debug!("reducing cell domains by cage-specific info");

    for cage in &puzzle.cages {
        match cage.operator {
            Operator::Add => {
                for &pos in &cage.cells {
                    let iter = cage.target - cage.cells.len() as i32 + 2..=puzzle.size as i32;
                    if let Some(n) = iter.next() {
                        affected_cells.insert(pos);
                        for n in iter {
                            cell_domains[pos].remove(n);
                        }
                    }
                }
            },
            Operator::Multiply => {
                let non_factors = (2..=puzzle.size as i32)
                    .filter(|n| !cage.target.is_multiple_of(n))
                    .collect::<Vec<_>>();
                if non_factors.is_empty() { return }
                for &pos in &cage.cells {
                    affected_cells.insert(pos);
                    for &n in &non_factors {
                        cell_domains[pos].remove(n);
                    }
                }
            },
            Operator::Subtract => {
                let size = puzzle.size as i32;
                if cage.target <= size / 2 { return }
                for &pos in &cage.cells {
                    affected_cells.insert(pos);
                    for n in size - cage.target + 1..=cage.target {
                        cell_domains[pos].remove(n);
                    }
                }
            },
            Operator::Divide => {
                let mut non_domain = CellDomain::with_all(puzzle.size);
                for n in 1..=puzzle.size as i32 / cage.target {
                    non_domain.remove(n);
                    non_domain.remove(n * cage.target);
                }
                if non_domain.is_empty() { return }
                for &pos in &cage.cells {
                    affected_cells.insert(pos);
                    for n in &non_domain {
                        cell_domains[pos].remove(n);
                    }
                }
            },
            Operator::Nop => {
                debug_assert_eq!(1, cage.cells.len());
                let pos = cage.cells[0];
                affected_cells.insert(pos);
                for n in (1..cage.target).chain(cage.target + 1..=puzzle.size as i32) {
                    cell_domains[pos].remove(n);
                }
            }
        }
    }
}
