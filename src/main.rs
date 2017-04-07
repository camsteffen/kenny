extern crate itertools;
extern crate rand;

mod board;
mod solve;

use board::*;
use solve::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    for _ in 1..2 {
        let size = 6;
        let board = generate_board(size);
        println!("Values:");
        print_square(&board.cells, board.size);
        println!("Cages Indices:");
        print_square(&board.cage_indices(), board.size);
        println!("Cages:");
        for (i, cage) in board.cages.iter().enumerate() {
            println!(" {:>2}: {} {}", i, operator_symbol(&cage.operator), cage.target);
        }
        solve(&board);
        stdout().flush().unwrap();
    }
}

