use std::collections::VecDeque;
use std::mem;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use crate::collections::square::{Square, SquareValue};
use crate::puzzle::Puzzle;
use crate::puzzle::{Cage, CellId, Solution, Value};
use crate::puzzle::{CageId, Operator};

const MAX_CAGE_SIZE: usize = 4;
const MAX_AVG_CAGE_SIZE: f32 = 2.2;
const CAGE_SIZE_DISTRIBUTION: f32 = 0.5;

type BorderId = u32;

pub fn generate_untested_puzzle(width: SquareValue) -> Puzzle {
    let (puzzle, _solution) = generate_untested_puzzle_with_solution(width);
    puzzle
}

// Note: The solution returned is not very important since the puzzle might have multiple solutions
pub fn generate_untested_puzzle_with_solution(width: SquareValue) -> (Puzzle, Solution) {
    let mut rng = thread_rng();
    let solution = random_latin_square(width, &mut rng);
    debug!("Solution:\n{}", &solution);
    let cage_cells = generate_cage_cells(width, &mut rng);
    let cages = cage_cells
        .into_iter()
        .map(|cells| {
            let values = cells.iter().map(|&i| solution[i]).collect::<Vec<_>>();
            let operator = random_operator(&values, &mut rng);
            let target = find_cage_target(operator, &values);
            Cage::new(cells, operator, target).unwrap()
        })
        .collect();
    let puzzle = Puzzle::new(width, cages).unwrap();
    (puzzle, solution)
}

fn random_latin_square(width: SquareValue, rng: &mut impl Rng) -> Square<Value> {
    let mut generate_seed = || {
        let mut seed = (0..width as i32).collect::<Vec<_>>();
        seed.shuffle(rng);
        seed
    };
    let seeds = [generate_seed(), generate_seed()];
    let mut square: Square<i32> = Square::with_width(width);
    for (i, row) in square.rows_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            *element = (seeds[0][i] + seeds[1][j]) % width as i32 + 1;
        }
    }
    square
}

fn shuffled_inner_borders(square_width: SquareValue, rng: &mut impl Rng) -> Vec<BorderId> {
    let num_borders = square_width * (square_width - 1) * 2;
    let mut borders = (0..num_borders).collect::<Vec<_>>();
    borders.shuffle(rng);
    borders
}

fn cells_touching_border(square_width: SquareValue, border_id: BorderId) -> (CellId, CellId) {
    let (width, border_id) = (square_width as CellId, border_id as CellId);
    let a = border_id / 2;
    let (a, b) = if border_id % 2 == 0 {
        (a, a + width)
    } else {
        let b = width - 1;
        let c = a / b * width + a % b;
        (c, c + 1)
    };
    (a, b)
}

fn generate_cage_cells(puzzle_width: SquareValue, rng: &mut impl Rng) -> Vec<Vec<CellId>> {
    let num_cells = (puzzle_width as usize).pow(2);
    let mut cage_map = Square::<CageId>::from_iter(0..num_cells).unwrap();
    let mut cages: Vec<Vec<CellId>> = (0..num_cells).map(|i| vec![i]).collect();
    let min_cage_count = (num_cells as f32 / MAX_AVG_CAGE_SIZE) as usize;
    let mut borders = VecDeque::from(shuffled_inner_borders(puzzle_width, rng));
    'target_cage_sizes: for target_cage_size in 2..=MAX_CAGE_SIZE {
        let border_count = (borders.len() as f32 * CAGE_SIZE_DISTRIBUTION) as usize;
        for _ in 0..border_count {
            let border_id = borders.pop_front().unwrap();
            let (cell1, cell2) = cells_touching_border(puzzle_width, border_id);
            let (mut cage_a, mut cage_b) = (cage_map[cell1], cage_map[cell2]);
            if cage_a > cage_b {
                mem::swap(&mut cage_a, &mut cage_b)
            }
            let cage_size = cages[cage_a].len() + cages[cage_b].len();
            if cage_size != target_cage_size {
                if cage_size > target_cage_size {
                    borders.push_back(border_id);
                }
                continue;
            }
            let a = cages.pop().unwrap();
            if cage_b == cages.len() {
                for &i in &a {
                    cage_map[i] = cage_a
                }
                cages[cage_a].extend(a);
            } else {
                for &i in &a {
                    cage_map[i] = cage_b
                }
                let b = mem::replace(&mut cages[cage_b], a);
                for &i in &b {
                    cage_map[i] = cage_a
                }
                cages[cage_a].extend(b);
            }
            if cages.len() == min_cage_count {
                break 'target_cage_sizes;
            }
        }
    }
    cages
}

fn random_operator(values: &[i32], rng: &mut impl Rng) -> Operator {
    if values.len() == 1 {
        return Operator::Nop;
    }
    let operators = possible_operators(values);
    *operators.choose(rng).unwrap()
}

fn possible_operators(values: &[i32]) -> Vec<Operator> {
    if values.len() < 2 {
        panic!("multiple values must be provided")
    }
    let mut operators = vec![Operator::Add, Operator::Multiply];
    if values.len() == 2 {
        operators.push(Operator::Subtract);
        let (min, max) = min_max(values);
        if max % min == 0 {
            operators.push(Operator::Divide);
        }
    }
    operators
}

fn find_cage_target(operator: Operator, values: &[Value]) -> Value {
    match operator {
        Operator::Add => values.iter().sum(),
        Operator::Subtract => {
            let (min, max) = min_max(values);
            max - min
        }
        Operator::Multiply => values.iter().product(),
        Operator::Divide => {
            let (min, max) = min_max(values);
            max / min
        }
        Operator::Nop => values[0],
    }
}

fn min_max<T>(slice: &[T]) -> (T, T)
where
    T: Copy + PartialOrd,
{
    let mut min = slice[0];
    let mut max = slice[0];
    for &e in &slice[1..] {
        if e < min {
            min = e
        }
        if e > max {
            max = e
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use crate::generate::generate_untested_puzzle;

    #[test]
    fn test() {
        for size in 3..=10 {
            for _ in 0..10 {
                // just test that it does not panic (such as from an invalid puzzle)
                let _ = generate_untested_puzzle(size);
            }
        }
    }
}
