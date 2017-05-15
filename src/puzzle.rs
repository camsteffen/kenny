use itertools::Itertools;
use itertools::repeat_call;
use rand::Rng;
use rand::thread_rng;
use solve::Solver;
use square::Coord;
use square::Square;

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
            for &j in &cage.cells {
                indices[j] = i;
            }
        }
        indices
    }

    pub fn generate(size: usize) -> Puzzle {
        let solution = random_latin_square(size);
        debug!("Solution:\n{}", &solution);
        let cages = generate_cages(&solution);
        Puzzle {
            size: size,
            cages: cages,
        }
    }

    pub fn solve(&self) -> Solver {
        let mut solver = Solver::new(self);
        solver.solve();
        solver
    }

}

fn generate_cages(cells: &Square<i32>) -> Vec<Cage> {
    let size = cells.size;
    let min_cage_size = 2;
    let max_cage_size = 4;
    let num_cells = cells.elements.len();
    let no_cage = -1;
    let mut cage_ids = Square::new(no_cage, cells.size);
    let mut uncaged = num_cells;
    let mut cur_cage = 0;
    let mut pos = Coord::from_index(0, size);
    let mut rng = thread_rng();
    //let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
    'cages: loop {
        let cage_size = rng.gen_range(min_cage_size, max_cage_size + 1);
        for _ in 0..cage_size {
            cage_ids[pos] = cur_cage;
            uncaged -= 1;
            if uncaged == 0 {
                break 'cages
            }
            let mut available_positions = Vec::with_capacity(4);
            for i in 0..2 {
                if pos[i] > 0 {
                    let mut available_pos = pos;
                    available_pos[i] -= 1;
                    available_positions.push(available_pos);
                }
                if pos[i] < size - 1 {
                    let mut available_pos = pos;
                    available_pos[i] += 1;
                    available_positions.push(available_pos);
                }
            }
            available_positions = available_positions.into_iter()
                .filter(|next| cage_ids[*next] == no_cage)
                .collect_vec();
            match rng.choose(&available_positions) {
                Some(p) => pos = *p,
                None => {
                    let index = cage_ids.elements.iter()
                        .position(|c| *c == no_cage)
                        .unwrap();
                    pos = Coord::from_index(index, size);
                    break
                }
            }
        }
        cur_cage += 1;
    }
    let num_cages = cur_cage as usize + 1;

    // for every cage_cells[i][j], cell j is in cage i
    let mut cage_cells = vec![Vec::new(); num_cages];
    for (cell, cage_index) in cage_ids.iter().map(|&i| i as usize).enumerate() {
        cage_cells[cage_index].push(cell);
    }
    let mut cages = Vec::with_capacity(num_cages);
    for cage_cells in cage_cells {
        let (operator, target) = find_cage_operator(cells, &cage_cells);
        cages.push(Cage {
            operator: operator,
            target: target,
            cells: cage_cells,
        });
    }
    cages
}

/**
 * Generate a random latin square with values from 1 to size
 */
fn random_latin_square(size: usize) -> Square<i32> {
    let mut rng = thread_rng();
    let seed = repeat_call(|| {
        let mut seed = (0..size as i32).collect_vec();
        rng.shuffle(&mut seed);
        seed
    }).take(2).collect_vec();
    let mut square: Square<i32> = Square::new(0, size);
    for (i, row) in square.rows_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            *element = (seed[0][i] + seed[1][j]) % size as i32 + 1;
        }
    }
    square
}

fn find_cage_operator(cells: &Square<i32>, indices: &[usize]) -> (Operator, i32) {
    let mut rng = thread_rng();
    let mut operators = Vec::with_capacity(4);
    let mut min: i32 = -1;
    let mut max: i32 = -1;
    let vals = indices.iter()
        .map(|&i| cells[i])
        .collect_vec();
    operators.push(Operator::Add);
    operators.push(Operator::Multiply);
    if indices.len() == 2 {
        min = *vals.iter().min().unwrap();
        max = *vals.iter().max().unwrap();
        operators.push(Operator::Subtract);
        if max % min == 0 {
            operators.push(Operator::Divide);
        }
    }
    let operator = rng.choose(&operators).unwrap().clone();
    let target = match operator {
        Operator::Add => vals.iter().sum(),
        Operator::Subtract => max - min,
        Operator::Multiply => vals.iter().product(),
        Operator::Divide => max / min,
    };
    (operator, target)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Operator { Add, Subtract, Multiply, Divide }

impl Operator {
    pub fn symbol(&self) -> char {
        match *self {
            Operator::Add      => '+',
            Operator::Subtract => '-',
            Operator::Multiply => '*',
            Operator::Divide   => '/',
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cage {
    pub operator: Operator,
    pub target: i32,
    pub cells: Vec<usize>,
}

/*
impl Cage {
    pub fn iter_cell_pos<'a>(&'a self, board_size: usize) -> Box<Iterator<Item=Coord> + 'a> {
        Box::new(self.cells.iter().map(move |i| Coord::from_index(*i, board_size)))
    }
}
*/

