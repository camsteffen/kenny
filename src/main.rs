extern crate itertools;
extern crate rand;
extern crate num;
#[macro_use] extern crate log;
extern crate env_logger;

mod board;
mod solve;

use board::*;
use solve::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    env_logger::init().unwrap();

    for _ in 1..200 {
        let size = 6;
        test(size);
    }
}

fn test(size: usize) {
    let (solution, cages) = generate_puzzle(size);
    println!("Values:");
    solution.print();
    println!("Cages Indices:");
    cage_indices(&cages, size).print();
    println!("Cages:");
    for (i, cage) in cages.iter().enumerate() {
        println!(" {:>2}: {} {}", i, operator_symbol(&cage.operator), cage.target);
    }
    let solve_soln = solve(&cages, size);
    println!("Solution:");
    solve_soln.print();
    let result = if solve_soln.elements.iter().any(|n| *n == 0) {
        "Fail"
    } else {
        "Success"
    };
    println!("Result: {}", result);
    stdout().flush().unwrap();
}
