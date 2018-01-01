use collections::square::SquareIndex;
use collections::FnvLinkedHashMap;

pub struct PuzzleMarkupChanges {
    pub cell_domain_value_removals: FnvLinkedHashMap<SquareIndex, Vec<i32>>,
    pub cell_solutions: Vec<(SquareIndex, i32)>,
}

impl PuzzleMarkupChanges {
    pub fn new() -> Self {
        Self {
            cell_domain_value_removals: FnvLinkedHashMap::default(),
            cell_solutions: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cell_domain_value_removals.clear();
        self.cell_solutions.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.cell_domain_value_removals.is_empty() && self.cell_solutions.is_empty()
    }

    pub fn remove_value_from_cell(&mut self, index: SquareIndex, value: i32) {
        self.cell_domain_value_removals.entry(index).or_insert_with(Vec::new).push(value);
    }

    pub fn solve_cell(&mut self, index: SquareIndex, value: i32) {
        self.cell_solutions.push((index, value));
    }
}
