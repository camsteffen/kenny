#![feature(retain_hash_collection)]

extern crate env_logger;
extern crate getopts;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate num;
extern crate png;
extern crate rand;
extern crate rusttype;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod cell_domain;
mod img;
mod puzzle;
mod solver;
mod square;
mod variable;

use getopts::Options;
use img::image;
use itertools::Itertools;
use log::LogLevel;
use puzzle::Puzzle;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::{stderr, stdin};

fn main() {
    env_logger::init().unwrap();

    do_main().unwrap_or_else(|e| {
        writeln!(stderr(), "{}", e).unwrap();
    });
}

struct Params {
    generate: Option<GenerateParams>,
    solve: bool,
    input_file: Option<String>,
    output_image: Option<String>,
}

struct GenerateParams {
    size: usize,
    output_file: Option<String>,
}

fn parse_args() -> Result<Params, String> {
    let args = env::args().collect_vec();

    let mut opts = Options::new();
    opts.optflag("h", "help", "show this help");
    opts.optflag("g", "generate", "generate a CamCam puzzle");
    opts.optflag("s", "solve", "attempt to solve the puzzle");
    opts.optopt("w", "size", "set the size of the puzzle", "5");
    opts.optopt("o", "output", "write the generated puzzle to a file", "puzzle.json");
    opts.optopt("", "image", "render a PNG image of the result", "puzzle.png");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string()),
    };

    if matches.opt_present("h") {
        return Err(opts.usage("Usage: camcam -g"));
    }

    let generate = if matches.opt_present("g") {
        let size = match matches.opt_str("w") {
            Some(s) => match s.parse() {
                Ok(s) => s,
                Err(e) => return Err(format!("Unable to parse size: {}", e)),
            },
            None => return Err("Please provide a size".to_string()),
        };
        let output_file = matches.opt_str("o");
        let params = GenerateParams {
            size: size,
            output_file: output_file,
        };
        Some(params)
    } else {
        None
    };
    
    let solve = matches.opt_present("s");

    let input_file = None;

    let output_image = matches.opt_str("image");

    let params = Params {
        generate: generate,
        solve: solve,
        input_file: input_file,
        output_image: output_image,
    };
    Ok(params)
}

fn read_puzzle_stdin() -> Puzzle {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf).unwrap();
    deserialize_puzzle(&buf)
}

fn deserialize_puzzle(s: &str) -> Puzzle {
    serde_json::from_str(s).unwrap_or_else(|e| {
        panic!("Unable to parse Puzzle: {}", e);
    })
}

fn do_main() -> Result<(), std::io::Error> {
    let params = match parse_args() {
        Ok(params) => params,
        Err(e) => {
            println!("{}", e);
            return Ok(())
        },
    };

    if params.input_file.is_some() && params.generate.is_some() {
        panic!("Cannot use input file and generate puzzle");
    }

    let puzzle =
        if let Some(path) = params.input_file {
            let mut file = File::open(path)?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            deserialize_puzzle(&buf)
        } else if let Some(gen_params) = params.generate {
            let puzzle = Puzzle::generate(gen_params.size);
            if let Some(path) = gen_params.output_file {
                let cages_json = serde_json::to_string(&puzzle).expect("serialize cages");
                let mut file = File::create(path)?;
                file.write_all(cages_json.into_bytes().as_slice())?;
            }
            puzzle
        } else {
            read_puzzle_stdin()
        };

    if log_enabled!(LogLevel::Info) {
        info!("Cage Indices:\n{}", puzzle.cage_map());
        info!("Cages:");
        for (i, cage) in puzzle.cages.iter().enumerate() {
            info!(" {:>2}: {} {}", i, &cage.operator.symbol(), cage.target);
        }
    }

    let markup = if params.solve {
        let markup = puzzle.solve();
        if log_enabled!(LogLevel::Info) {
            let result = if markup.solved() {
                "Fail"
            } else {
                "Success"
            };
            info!("Result: {}", result);
        }
        Some(markup)
    } else {
        None
    };

    if let Some(path) = params.output_image {
        image(&puzzle, markup.as_ref(), &path)?;
    }

    Ok(())
}

