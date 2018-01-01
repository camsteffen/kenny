use collections::Square;
use collections::square::SquareIndex;
use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use rand::Rng;
use rand::thread_rng;

/// Generate a random puzzle of a certain size
pub fn generate_puzzle(size: usize) -> Puzzle {
    let solution = random_latin_square(size);
    debug!("Solution:\n{}", &solution);
    let cages = generate_cages(&solution);
    Puzzle::new(size, cages)
}
    
// TODO investigate another method - randomly remove cell borders
fn generate_cages(square: &Square<i32>) -> Vec<Cage> {
    let width = square.width();
    let min_cage_size = 2;
    let max_cage_size = 4;
    let num_cells = square.len();
    let no_cage = -1;
    let mut cage_ids = Square::with_width_and_value(width, no_cage);
    let mut uncaged = num_cells;
    let mut cur_cage = 0;
    let mut pos = SquareIndex(0).as_coord(width);
    let mut rng = thread_rng();
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
                if pos[i] < width - 1 {
                    let mut available_pos = pos;
                    available_pos[i] += 1;
                    available_positions.push(available_pos);
                }
            }
            available_positions = available_positions.into_iter()
                .filter(|next| cage_ids[*next] == no_cage)
                .collect::<Vec<_>>();
            match rng.choose(&available_positions) {
                Some(p) => pos = *p,
                None => {
                    let index = cage_ids.iter()
                        .position(|c| *c == no_cage)
                        .unwrap();
                    pos = SquareIndex(index).as_coord(width);
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
        cage_cells[cage_index].push(SquareIndex(cell));
    }
    let mut cages = Vec::with_capacity(num_cages);
    for cells in cage_cells {
        let (operator, target) = find_cage_operator(square, &cells);
        cages.push(Cage {
            operator,
            target,
            cells,
        });
    }
    cages
}

/**
 * Generate a random latin square with values from 1 to size
 */
fn random_latin_square(size: usize) -> Square<i32> {
    let mut rng = thread_rng();
    let mut generate_seed = || {
        let mut seed = (0..size as i32).collect::<Vec<_>>();
        rng.shuffle(&mut seed);
        seed
    };
    let seeds = [generate_seed(), generate_seed()];
    let mut square: Square<i32> = Square::with_width_and_value(size, 0);
    for (i, row) in square.rows_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            *element = (seeds[0][i] + seeds[1][j]) % size as i32 + 1;
        }
    }
    square
}

/// Selects a random, valid operator for a cage
fn find_cage_operator(cells: &Square<i32>, indices: &[SquareIndex]) -> (Operator, i32) {
    let mut rng = thread_rng();
    let mut operators = Vec::with_capacity(4);
    let mut min: i32 = -1;
    let mut max: i32 = -1;
    let vals = indices.iter()
        .map(|&i| cells[i])
        .collect::<Vec<_>>();
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
        Operator::Nop => unreachable!(),
    };
    (operator, target)
}
