extern crate itertools;
extern crate png;

mod square;
mod solve;
mod board;
mod img;

extern crate num;
extern crate rusttype;
extern crate rand;
#[macro_use]
extern crate log;
extern crate env_logger;

use img::image;
use std::env::args;
use board::*;
use solve::*;

fn main() {
    env_logger::init().unwrap();

    let size = match args().nth(1) {
        Some(size) => size.parse().unwrap(),
        None => 5,
    };

    test(size);
}

fn test(size: usize) {
    let (solution, cages) = generate_puzzle(size);
    println!("Values:");
    solution.print();
    println!("Cage Indices:");
    cage_map(&cages, size).print();
    println!("Cages:");
    for (i, cage) in cages.iter().enumerate() {
        println!(" {:>2}: {} {}", i, operator_symbol(&cage.operator), cage.target);
    }
    let markup = solve(&cages, size);
    /*
    println!("Solution:");
    solve_soln.print();
    */
    let result = if markup.solved() {
        "Fail"
    } else {
        "Success"
    };
    println!("Result: {}", result);

    image(&cages, &markup, size);
}
