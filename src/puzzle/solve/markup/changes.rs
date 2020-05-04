use ahash::AHashMap;
use crate::puzzle::{CellId, Value, CageId};

#[derive(Default)]
pub struct PuzzleMarkupChanges {
    pub cell_domain_value_removals: AHashMap<CellId, Vec<Value>>,
    pub cell_solutions: Vec<(CellId, Value)>,
    pub cage_solution_removals: AHashMap<CageId, Vec<usize>>,
}

impl PuzzleMarkupChanges {
    pub fn clear(&mut self) {
        self.cell_domain_value_removals.clear();
        self.cell_solutions.clear();
        self.cage_solution_removals.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.cell_domain_value_removals.is_empty()
            && self.cell_solutions.is_empty()
            && self.cage_solution_removals.is_empty()
    }

    pub fn remove_value_from_cell(&mut self, id: CellId, value: i32) {
        self.cell_domain_value_removals.entry(id)
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn solve_cell(&mut self, id: CellId, value: i32) {
        self.cell_solutions.push((id, value));
    }

    pub fn remove_cage_solution(&mut self, cage_id: CageId, solution_index: usize) {
        self.cage_solution_removals.entry(cage_id)
            .or_insert_with(Vec::new)
            .push(solution_index);
    }
}
