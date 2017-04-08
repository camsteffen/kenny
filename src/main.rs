extern crate itertools;
extern crate rand;
extern crate num;

mod board;
mod solve;

use board::*;
use solve::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    for _ in 1..2 {
        let size = 6;
        let (solution, cages) = generate_puzzle(size);
        println!("Values:");
        solution.print();
        println!("Cages Indices:");
        cage_indices(&cages, size).print();
        println!("Cages:");
        for (i, cage) in cages.iter().enumerate() {
            println!(" {:>2}: {} {}", i, operator_symbol(&cage.operator), cage.target);
        }
        solve(&cages, size);
        stdout().flush().unwrap();
    }
}

