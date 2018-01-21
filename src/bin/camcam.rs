extern crate camcam;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate clap;

use camcam::puzzle::Puzzle;
use camcam::puzzle;
use log::LogLevel;
use std::fs::File;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const DEFAULT_WIDTH: u32 = 4;
const DEFAULT_PATH: &str = "output";
const IMG_EXT: &str = "jpg";

fn main() {
    env_logger::init().unwrap();

    let matches = clap::App::new("CamCam")
        .author("Cameron Steffen <cam.steffen94@gmail.com>")
        .about("Solve KenKen Puzzles")
        .group(clap::ArgGroup::with_name("source")
            .args(&["generate", "input"])
            .required(true))
        .arg(clap::Arg::with_name("generate")
            .short("g")
            .long("generate")
            .help("generate KenKen puzzle(s)"))
        .arg(clap::Arg::with_name("input")
            .short("i")
            .long("input")
            .takes_value(true)
            .value_name("PATH")
            .help("read a KenKen puzzle from a file"))
        .arg(clap::Arg::with_name("solve")
            .short("s")
            .long("solve")
            .help("solve KenKen puzzle(s)"))
        .arg(clap::Arg::with_name("width")
            .short("w")
            .long("width")
            .takes_value(true)
            .value_name("WIDTH")
            .requires("generate")
            .help("set the width and height of the generated puzzle"))
        .arg(clap::Arg::with_name("output")
            .long("output_path")
            .help("directory to save files"))
        .arg(clap::Arg::with_name("count")
            .short("c")
            .long("count")
            .requires("generate")
            .takes_value(true)
            .help("the number of puzzles to generate (and solve)"))
        .arg(clap::Arg::with_name("save_puzzle")
            .long("save-puzzle")
            .requires("generate")
            .help("save the puzzle to a file"))
        .arg(clap::Arg::with_name("save_image")
            .long("save-image")
            .help("save an image of the puzzle(s)"))
        .arg(clap::Arg::with_name("save_solved_image")
            .long("save-solved-image")
            .value_name("PATH")
            .requires("solve")
            .help("save an image of the solved (or partially solved) puzzle(s)"))
        .arg(clap::Arg::with_name("step_images")
            .long("step-images")
            .value_name("DIRECTORY")
            .help("save an image of the puzzle at each step of the solving process"))
        .get_matches();

    let Context { output_path, source, solve, save_image } = build_context(&matches);

    let handle_puzzle = |puzzle: &Puzzle, path: &Path| -> bool {
        print_puzzle(puzzle);
        if save_image {
            if !save_puzzle_image(puzzle, path) {
                return false
            }
        }
        if let Some(SolveContext { save_image: save_solved_image, step_images }) = solve {
            if !solve_puzzle(puzzle, path, save_solved_image, step_images) {
                return false
            }
        }
        true
    };

    let output_path = Path::new(output_path.unwrap_or(DEFAULT_PATH));

    match source {
        SourceContext::File(path) => {
            if let Some(puzzle) = input_puzzle(Path::new(path)) {
                handle_puzzle(&puzzle, output_path);
            }
        },
        SourceContext::Generate(GenerateContext { count, width, save_puzzle }) => {
            let handle_generated_puzzle = |puzzle: &Puzzle, output_path: &Path| -> bool {
                let save_success = !save_puzzle || ::save_puzzle(&puzzle, output_path);
                save_success && handle_puzzle(puzzle, output_path)
            };

            if count == 1 {
                let puzzle = puzzle::generate_puzzle(width);
                handle_generated_puzzle(&puzzle, output_path);
            } else {
                generate_puzzles(count, width, output_path, handle_generated_puzzle);
            }
        },
    }
}

fn input_puzzle(path: &Path) -> Option<Puzzle> {
    println!("reading puzzle from \"{}\"", path.display());
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("could not open puzzle file at \"{}\": {}", path.display(), e);
            return None
        },
    };
    let mut buf = String::new();
    if let Err(e) = file.read_to_string(&mut buf) {
        eprintln!("could not read puzzle: {}", e);
        return None
    }
    let puzzle = match puzzle::parse(&buf) {
        Ok(puzzle) => puzzle,
        Err(e) => {
            eprintln!("could not parse puzzle: {}", e);
            return None
        },
    };
    Some(puzzle)
}

fn generate_puzzles<F: Fn(&Puzzle, &Path) -> bool>(count: u32, width: u32, output_path: &Path, consumer: F) {
    //let path = Path::new(path.unwrap_or(DEFAULT_PATH));
    let root = output_path.to_path_buf();
    for i in 0..count {
        println!("Generating puzzle {}/{}", i + 1, count);
        let puzzle = puzzle::generate_puzzle(width);
        let mut path = root.clone();
        path.push(format!("puzzle_{}", i + 1));
        if let Err(e) = fs::create_dir(&path) {
            eprintln!("unable to create directory \"{}\": {}", path.display(), e);
            return
        }
        if !consumer(&puzzle, &path) { return }
    }
}

fn save_puzzle(puzzle: &Puzzle, output_path: &Path) -> bool {
    let mut path = output_path.to_path_buf();
    path.push("puzzle");
    println!("Saving puzzle to \"{}\"", path.display());
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("could not open file at \"{}\": {}", path.display(), e);
            return false
        },
    };
    if let Err(e) = file.write_all(&puzzle.to_string().into_bytes()) {
        eprintln!("could not write puzzle to \"{}\": {}", path.display(), e);
        return false
    }
    true
}

fn print_puzzle(puzzle: &Puzzle) {
    if log_enabled!(LogLevel::Info) {
        info!("Cage Indices:\n{}", puzzle.cage_map);
        info!("Cages:");
        for (i, cage) in puzzle.cages.iter().enumerate() {
            info!(" {:>2}: {} {}", i, &cage.operator.symbol().unwrap_or(' '), cage.target);
        }
    }
}

fn save_puzzle_image(puzzle: &Puzzle, output_path: &Path) -> bool {
    let mut path = output_path.to_path_buf();
    path.push(format!("image.{}", IMG_EXT));
    println!("Saving puzzle image to \"{}\"", path.display());
    let image = puzzle.image();
    if let Err(e)  = image.save(&path) {
        eprintln!("could not save puzzle image to \"{}\": {}", path.display(), e);
        return false
    }
    true
}

fn solve_puzzle(puzzle: &Puzzle, output_path: &Path, save_solved_image: bool, step_images: bool) -> bool {
    let step_images_path = if step_images {
        let mut path = output_path.to_path_buf();
        path.push("steps");
        Some(path)
    } else {
        None
    };
    let markup = puzzle.solve(step_images_path.as_ref().map(PathBuf::as_path));
    let result = if markup.is_solved() { "Success" } else { "Fail" };
    println!("Result: {}", result);

    if save_solved_image {
        let image = puzzle.image_with_markup(&markup);
        let mut path = output_path.to_path_buf();
        path.push(format!("image_solved.{}", IMG_EXT));
        if let Err(e) = image.save(&path) {
            eprintln!("could not save image to \"{}\": {}", path.display(), e);
            return false
        }
    }

    true
}

fn build_context<'a>(matches: &'a clap::ArgMatches) -> Context<'a> {
    Context {
        output_path: matches.value_of("output_path"),
        source: match matches.value_of("input") {
            Some(path) => SourceContext::File(path),
            None => SourceContext::Generate(GenerateContext {
                count: matches.value_of("count")
                    .map(|s| s.parse::<u32>().expect("invalid count"))
                    .unwrap_or(1),
                width: matches.value_of("width")
                    .map(|s| s.parse::<u32>().expect("invalid width"))
                    .unwrap_or(DEFAULT_WIDTH),
                save_puzzle: matches.is_present("save_puzzle"),
            }),
        },
        solve: match matches.is_present("solve") {
            true => Some(SolveContext {
                save_image: matches.is_present("save_solved_image"),
                step_images: matches.is_present("step_images"),
            }),
            false => None,
        },
        save_image: matches.is_present("save_image"),
    }
}

struct Context<'a> {
    output_path: Option<&'a str>,
    source: SourceContext<'a>,
    solve: Option<SolveContext>,
    save_image: bool,
}

enum SourceContext<'a> {
    File(&'a str),
    Generate(GenerateContext),
}

struct GenerateContext {
    count: u32,
    width: u32,
    save_puzzle: bool,
}

struct SolveContext {
    save_image: bool,
    step_images: bool,
}