use collections::square::SquareIndex;

pub struct DomainChangeSet {
    cell_domain_value_removals: Vec<CellDomainValueRemoval>,
}

impl DomainChangeSet {
    pub fn remove_value_from_cell(&mut self, index: SquareIndex, value: i32) {
        self.cell_domain_value_removals.push(CellDomainValueRemoval { index, value });
    }
}

struct CellDomainValueRemoval {
    index: SquareIndex,
    value: i32,
}