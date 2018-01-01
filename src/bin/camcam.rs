extern crate camcam;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate clap;

use log::LogLevel;
use camcam::puzzle;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::stderr;

const DEFAULT_SIZE: usize = 4;

fn main() {
    env_logger::init().unwrap();

    do_main().unwrap_or_else(|e| {
        writeln!(stderr(), "{}", e).unwrap();
    });
}

/*
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
*/

/*
fn read_puzzle_stdin() -> Puzzle {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf).unwrap();
    parse_puzzle(&buf)
}
*/

fn do_main() -> Result<(), std::io::Error> {
    let matches = clap::App::new("CamCam")
            .author("Cameron Steffen <cam.steffen94@gmail.com>")
            .about("Solve KenKen Puzzles")
            .arg(clap::Arg::with_name("generate")
                    .short("g")
                    .help("Generates a KenKen puzzle"))
            .arg(clap::Arg::with_name("solve")
                    .short("s")
                    .help("Solves a KenKen puzzle"))
            .arg(clap::Arg::with_name("input")
                    .short("i")
                    .takes_value(true)
                    .value_name("FILE")
                    .help("Reads a KenKen puzzle from a file"))
            .arg(clap::Arg::with_name("size")
                    .short("w")
                    .takes_value(true)
                    .value_name("SIZE")
                    .help("Sets the width/height for the generated puzzle"))
            .arg(clap::Arg::with_name("output_image")
                    .short("o")
                    .takes_value(true)
                    .value_name("OUT_IMAGE")
                    .help("Writes the puzzle to a PNG image"))
            .get_matches();

    /*
    let params = match parse_args() {
        Ok(params) => params,
        Err(e) => {
            println!("{}", e);
            return Ok(())
        },
    };
    */

    if matches.is_present("input") && matches.is_present("generate") {
        panic!("Cannot use input file and generate puzzle");
    }

    let puzzle =
        if let Some(path) = matches.value_of("input") {
            let mut file = File::open(path)?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            puzzle::parse(&buf).unwrap_or_else(|e| panic!("Error parsing puzzle from {}: {}", path, e))
        } else if matches.is_present("generate") {
            let size = matches.value_of("size")
                .map(|s| s.parse::<usize>().expect("invalid size"))
                .unwrap_or(DEFAULT_SIZE);
            let puzzle = puzzle::generate(size);
            if let Some(path) = matches.value_of("output_file") {
                // let cages_json = serde_json::to_string(&puzzle).expect("serialize cages");
                let cages_json = String::new();
                let mut file = File::create(path)?;
                file.write_all(cages_json.into_bytes().as_slice())?;
            }
            puzzle
        } else {
            panic!("nothing to do")
        };

    if log_enabled!(LogLevel::Info) {
        info!("Cage Indices:\n{}", puzzle.cage_map);
        info!("Cages:");
        for (i, cage) in puzzle.cages.iter().enumerate() {
            info!(" {:>2}: {} {}", i, &cage.operator.symbol().unwrap_or(' '), cage.target);
        }
    }

    let markup =
        if matches.is_present("solve") {
            let markup = puzzle.solve();
            if log_enabled!(LogLevel::Info) {
                //let result = if markup { "Success" } else { "Fail" };
                //info!("Result: {}", result);
            }
            Some(markup)
        } else {
            None
        };

    if let Some(path) = matches.value_of("output_image") {
        let image = match markup {
            Some(markup) => puzzle.image_with_markup(&markup),
            None => puzzle.image(),
        };
        image.save(path)?;
    }

    Ok(())
}

