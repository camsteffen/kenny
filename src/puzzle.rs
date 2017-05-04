use board::*;

#[derive(Deserialize, Serialize)]
pub struct Puzzle {
    pub size: usize,
    pub cages: Vec<Cage>,
}
