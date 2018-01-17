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
        .arg(clap::Arg::with_name("step_images")
            .long("step-images")
            .help("Saves an image for each step of the solving process"))
        .get_matches();

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
            let puzzle = puzzle::generate_puzzle(size);
            if let Some(path) = matches.value_of("output_file") {
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
            let save_step_images = matches.is_present("step_images");
            let markup = puzzle.solve(save_step_images);
            if log_enabled!(LogLevel::Info) {
                let result = if markup.is_solved() { "Success" } else { "Fail" };
                info!("Result: {}", result);
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

