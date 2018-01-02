use collections::square::SquareIndex;
use fnv::FnvHashMap;

pub struct PuzzleMarkupChanges {
    pub cell_domain_value_removals: FnvHashMap<SquareIndex, Vec<i32>>,
    pub cell_solutions: Vec<(SquareIndex, i32)>,
    pub cage_solution_removals: FnvHashMap<usize, Vec<usize>>,
}

impl PuzzleMarkupChanges {
    pub fn new() -> Self {
        Self {
            cell_domain_value_removals: FnvHashMap::default(),
            cell_solutions: Vec::new(),
            cage_solution_removals: FnvHashMap::default(),
        }
    }

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

    pub fn remove_value_from_cell(&mut self, index: SquareIndex, value: i32) {
        self.cell_domain_value_removals.entry(index).or_insert_with(Vec::new).push(value);
    }

    pub fn solve_cell(&mut self, index: SquareIndex, value: i32) {
        self.cell_solutions.push((index, value));
    }

    pub fn remove_cage_solution(&mut self, cage_index: usize, solution_index: usize) {
        self.cage_solution_removals.entry(cage_index).or_insert_with(Vec::new).push(solution_index);
    }
}
