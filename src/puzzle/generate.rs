use collections::Square;
use collections::square::SquareIndex;
use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use rand::Rng;
use rand::thread_rng;
use std::mem;

const MAX_CAGE_SIZE: usize = 4;

pub fn generate_puzzle(width: usize) -> Puzzle {
    let (puzzle, _) = generate_puzzle_with_solution(width);
    puzzle
}

pub fn generate_puzzle_with_solution(width: usize) -> (Puzzle, Square<i32>) {
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

fn random_latin_square(width: usize) -> Square<i32> {
    let mut rng = thread_rng();
    let mut generate_seed = || {
        let mut seed = (0..width as i32).collect::<Vec<_>>();
        rng.shuffle(&mut seed);
        seed
    };
    let seeds = [generate_seed(), generate_seed()];
    let mut square: Square<i32> = Square::with_width_and_value(width, 0);
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

fn cells_touching_border(square_width: usize, border_id: usize) -> (usize, usize) {
    let a = border_id / 2;
    if border_id % 2 == 0 {
        (a, a + square_width)
    } else {
        let b = square_width - 1;
        let c = a / b * square_width + a % b;
        (c, c + 1)
    }
}

fn generate_cage_cells(puzzle_width: usize) -> Vec<Vec<SquareIndex>> {
    let num_cells = puzzle_width.pow(2);
    let mut cage_map = (0..num_cells).collect::<Vec<_>>();
    let mut cages = (0..num_cells).map(|i| vec![i]).collect::<Vec<_>>();
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
    cages.into_iter().map(|cells| cells.into_iter().map(|i| SquareIndex(i)).collect()).collect()
}

fn generate_cage_cells_snake(square: &Square<i32>) -> Vec<Vec<SquareIndex>> {
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
    cage_cells
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
    let mut max = slice[1];
    for &e in &slice[1..] {
        if e < min { min = e }
        if e > max { max = e }
    }
    (min, max)
}

