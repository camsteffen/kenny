use collections::Square;
use collections::square::SquareIndex;
use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use rand::Rng;
use rand::thread_rng;
use std::mem;

const MAX_CAGE_SIZE: usize = 4;

pub fn generate_puzzle(width: u32) -> Puzzle {
    let (puzzle, _) = generate_puzzle_with_solution(width);
    puzzle
}

pub fn generate_puzzle_with_solution(width: u32) -> (Puzzle, Square<i32>) {
    let solution = random_latin_square(width);
    debug!("Solution:\n{}", &solution);
    let cage_cells = generate_cage_cells(width);
    let cages = cage_cells.into_iter().map(|cells| {
        let values = cells.iter().map(|&i| solution[i]).collect::<Vec<_>>();
        let operator = random_operator(&values);
        let target = find_cage_target(operator, &values);
        Cage { cells, operator, target }
    }).collect();
    let puzzle = Puzzle::new(width, cages);
    (puzzle, solution)
}

fn random_latin_square(width: u32) -> Square<i32> {
    let mut rng = thread_rng();
    let mut generate_seed = || {
        let mut seed = (0..width as i32).collect::<Vec<_>>();
        rng.shuffle(&mut seed);
        seed
    };
    let seeds = [generate_seed(), generate_seed()];
    let mut square: Square<i32> = Square::with_width_and_value(width as usize, 0);
    for (i, row) in square.rows_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            *element = (seeds[0][i] + seeds[1][j]) % width as i32 + 1;
        }
    }
    square
}

fn shuffled_inner_borders(square_width: usize) -> Vec<usize> {
    let mut rng = thread_rng();
    let num_borders = square_width * (square_width - 1) * 2;
    let mut borders = (0..num_borders).collect::<Vec<_>>();
    rng.shuffle(&mut borders);
    borders
}

fn cells_touching_border(square_width: usize, border_id: usize) -> (SquareIndex, SquareIndex) {
    let a = border_id / 2;
    let (a, b) = if border_id % 2 == 0 {
        (a, a + square_width)
    } else {
        let b = square_width - 1;
        let c = a / b * square_width + a % b;
        (c, c + 1)
    };
    (SquareIndex(a), SquareIndex(b))
}

fn generate_cage_cells(puzzle_width: u32) -> Vec<Vec<SquareIndex>> {
    let puzzle_width = puzzle_width as usize;
    let num_cells = puzzle_width.pow(2);
    let cage_map = (0..num_cells).collect::<Vec<_>>();
    let mut cage_map = Square::from_vec(cage_map).unwrap();
    let mut cages = (0..num_cells).map(|i| vec![SquareIndex(i)]).collect::<Vec<_>>();
    for border_id in shuffled_inner_borders(puzzle_width) {
        let (cell1, cell2) = cells_touching_border(puzzle_width, border_id);
        let (mut cage1, mut cage2) = (cage_map[cell1], cage_map[cell2]);
        if cage1 > cage2 { mem::swap(&mut cage1, &mut cage2) }
        let cage_size = cages[cage1].len() + cages[cage2].len();
        if cage_size > MAX_CAGE_SIZE { continue }
        let a = cages.pop().unwrap();
        if cage2 == cages.len() {
            for &i in &a { cage_map[i] = cage1 }
            cages[cage1].extend(a);
        } else {
            for &i in &a { cage_map[i] = cage2 }
            let b = mem::replace(&mut cages[cage2], a);
            for &i in &b { cage_map[i] = cage1 }
            cages[cage1].extend(b);
        }
    }
    cages
}

fn random_operator(values: &[i32]) -> Operator {
    if values.len() == 1 { return Operator::Nop }
    let mut rng = thread_rng();
    let operators = possible_operators(values);
    *rng.choose(&operators).unwrap()
}

fn possible_operators(values: &[i32]) -> Vec<Operator> {
    if values.len() < 2 { panic!("multiple values must be provided") }
    let mut operators = vec![Operator::Add, Operator::Multiply];
    if values.len() == 2 {
        operators.push(Operator::Subtract);
        let (min, max) = min_max(&values);
        if max % min == 0 {
            operators.push(Operator::Divide);
        }
    }
    operators
}

fn find_cage_target(operator: Operator, values: &[i32]) -> i32 {
    match operator {
        Operator::Add => values.iter().sum(),
        Operator::Subtract => {
            let (min, max) = min_max(values);
            max - min
        },
        Operator::Multiply => values.iter().product(),
        Operator::Divide => {
            let (min, max) = min_max(values);
            max / min
        },
        Operator::Nop => values[0],
    }
}

fn min_max<T>(slice: &[T]) -> (T, T) where T: Copy + PartialOrd {
    let mut min = slice[0];
    let mut max = slice[0];
    for &e in &slice[1..] {
        if e < min { min = e }
        if e > max { max = e }
    }
    (min, max)
}

