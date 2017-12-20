use collections::RangeSet;

struct VectorValueConstraint {
    data: Vec<Vec<RangeSet>>,
}

impl VectorValueConstraint {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![vec![RangeSet::with_all(size); size]; 2 * size],
        }
    }
}