use board::*;
use square::*;

#[derive(Deserialize, Serialize)]
pub struct Puzzle {
    pub size: usize,
    pub cages: Vec<Cage>,
}

impl Puzzle {
    /**
     * Create a square of values where each value represents the index of the cage
     * containing that position
     */
    pub fn cage_map(&self) -> Square<usize> {
        let mut indices = Square::new(0, self.size);
        for (i, cage) in self.cages.iter().enumerate() {
            for j in cage.cells.iter() {
                indices[*j] = i;
            }
        }
        indices
    }
}
