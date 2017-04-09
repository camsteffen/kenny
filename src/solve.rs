use board::*;

#[derive(Clone)]
struct CellMarkup {
    count: usize,
    possible: Vec<bool>,
}

impl CellMarkup {
    fn new(size: usize) -> CellMarkup {
        CellMarkup {
            count: size,
            possible: vec![true; size],
        }
    }

    fn set_possible(&mut self, n: usize, possible: bool) {
        if self.possible[n] != possible {
            self.possible[n] = possible;
            self.count = if possible { self.count + 1 } else { self.count - 1 }
        }
    }

    fn set_value(&mut self, n: i32) {
        for (i, p) in self.possible.iter_mut().enumerate() {
            *p = i as i32 == n;
        }
        self.count = 1;
    }

    fn value(&self) -> Option<i32> {
        match self.count {
            1 => Some(self.possible.iter().position(|p| *p).unwrap() as i32),
            _ => None,
        }
    }
}

type Markup = Square<CellMarkup>;

pub fn solve(cages: &Vec<Cage>, size: usize) -> Board {
    let mut markup = Square::new(CellMarkup::new(size), size);
    
    // clear board
    let mut board = Square::new(0, size);

    for cage_index in 0..cages.len() {
        solve_cage(&board, &cages, &mut markup, cage_index);
    }

    for (pos, m) in markup.iter() {
        board[&pos] = m.value().unwrap_or(0);
    }

    board
}

fn solve_cage(board: &Board, cages: &Vec<Cage>, markup: &mut Markup, cage_index: usize) {
    let cage = &cages[cage_index];
    if cage.cells.len() == 1 {
        let cell_index = cage.cells[0];
        markup.elements[cell_index].set_value(cage.target);
    }
}
