use square::vector::VectorId;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone)]
pub struct CageMarkup {
    pub vector_vals: HashMap<VectorId, HashSet<i32>>,
}

impl CageMarkup {
    pub fn new() -> CageMarkup {
        CageMarkup {
            vector_vals: HashMap::new(),
        }
    }
}
