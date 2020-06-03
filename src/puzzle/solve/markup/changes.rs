use crate::puzzle::{CageId, CellId, Value};
use ahash::AHashMap;
use std::collections::hash_map::Entry;

#[derive(Clone, Debug, PartialEq)]
pub enum CellChange {
    DomainRemovals(Vec<Value>),
    Solution(Value),
}

#[derive(Debug, Default, PartialEq)]
pub struct PuzzleMarkupChanges {
    pub cells: AHashMap<CellId, CellChange>,
    pub cage_solution_removals: AHashMap<CageId, Vec<usize>>,
}

impl PuzzleMarkupChanges {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.cage_solution_removals.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty() && self.cage_solution_removals.is_empty()
    }

    pub fn cell_domain_removals(&self) -> impl Iterator<Item = (CellId, &[Value])> {
        self.cells.iter().filter_map(|(&id, e)| match e {
            CellChange::DomainRemovals(values) => Some((id, values.as_slice())),
            _ => None,
        })
    }

    pub fn cell_solutions<'a>(&'a self) -> impl Iterator<Item = (CellId, Value)> + 'a {
        self.cells.iter().filter_map(|(&id, e)| match e {
            CellChange::Solution(value) => Some((id, *value)),
            _ => None,
        })
    }

    pub fn remove_value_from_cell(&mut self, id: CellId, value: i32) {
        match self.cells.entry(id) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                // already solved, ignore
                CellChange::Solution(_) => return,
                CellChange::DomainRemovals(values) => values.push(value),
            },
            Entry::Vacant(entry) => {
                entry.insert(CellChange::DomainRemovals(vec![value]));
            }
        };
    }

    pub fn solve_cell(&mut self, id: CellId, value: i32) {
        self.cells.insert(id, CellChange::Solution(value));
    }

    pub fn remove_cage_solution(&mut self, cage_id: CageId, solution_index: usize) {
        self.cage_solution_removals
            .entry(cage_id)
            .or_default()
            .push(solution_index);
    }
}
