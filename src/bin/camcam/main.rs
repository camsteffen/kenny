#![feature(generators, generator_trait)]

extern crate camcam;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate clap;
extern crate tempfile;
extern crate image;

mod puzzle_temp_dir;

use puzzle_temp_dir::PuzzleTempDir;
use camcam::gen_utils::gen_to_iter;
use camcam::puzzle::Puzzle;
use camcam::puzzle;
use camcam::puzzle::solve::PuzzleSolver;
use camcam::puzzle::solve::PuzzleMarkup;
use camcam::puzzle::solve::SolveError;
use log::LogLevel;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io;
use std::path::PathBuf;
use std::fmt::Display;
use std::fmt;
use tempfile::TempDir;
use camcam::puzzle::PuzzleImageBuilder;

type Result<T> = std::result::Result<T, Error>;

const DEFAULT_WIDTH: u32 = 4;
const DEFAULT_PATH: &str = "output";

fn main() {
    env_logger::init().unwrap();

    let matches = clap::App::new("CamCam")
        .author("Cameron Steffen <cam.steffen94@gmail.com>")
        .about("Solve KenKen Puzzles")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
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
            .long("output-path")
            .help("directory to save files"))
        .arg(clap::Arg::with_name("count")
            .short("c")
            .long("count")
            .requires("generate")
            .takes_value(true)
            .help("the number of puzzles to generate (and solve)"))
        .arg(clap::Arg::with_name("unsolvable")
            .long("unsolvable")
            .requires("generate")
            .help("include unsolvable generated puzzles"))
        .arg(clap::Arg::with_name("save_puzzle")
            .long("save-puzzle")
            .requires("generate")
            .help("save the puzzle to a file"))
        .arg(clap::Arg::with_name("save_image")
            .long("save-image")
            .help("save an image of the puzzle(s)"))
        .arg(clap::Arg::with_name("save_solved_image")
            .long("save-solved-image")
            .requires("solve")
            .help("save an image of the solved (or partially solved) puzzle(s)"))
        .arg(clap::Arg::with_name("save_step_images")
            .long("save-step-images")
            .help("save an image of the puzzle at each step of the solving process"))
        .arg(clap::Arg::with_name("image_width")
            .long("image-width")
            .takes_value(true)
            .value_name("PIXELS")
            .help("sets the approx. width of saved images in pixels"))
        .get_matches();

    let context = build_context(&matches);

    match do_main(&context) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}

fn do_main(context: &Context) -> Result<()> {
    let output_dir = get_output_dir(context)?;
    let puzzles = get_puzzles_iter(context);
    let mut paths = output_dir.map(puzzle_path_iterator);
    let mut puzzle_handler = PuzzleHandler::new(context);
    let write_limit = match &context.source {
        &SourceContext::Generate(GenerateContext { count, .. }) => count,
        _ => u32::max_value(),
    };
    let mut write_count = 0;
    for puzzle in puzzles {
        let puzzle = puzzle?;
        if let Some(dir) = puzzle_handler.handle_puzzle(&puzzle)? {
            let temp_dir: TempDir = dir.into();
            let path = temp_dir.into_path();
            let target_path = paths.as_mut().unwrap().next().unwrap();
            fs::rename(path, &target_path)?;
            println!("Saving puzzle to {}", target_path.display());
            write_count += 1;
            if write_count == write_limit {
                break
            }
        }
    }
    Ok(())
}

fn get_output_dir<'a>(context: &'a Context) -> Result<Option<&'a Path>> {
    let path = if context.save_any() {
        let path = Path::new(context.output_path.unwrap_or(DEFAULT_PATH));
        if !path.exists() {
            fs::create_dir(path).map_err(|e| Error::CreateDirectory(path.to_path_buf(), e))?;
        }
        Some(path)
    } else if context.output_path.is_some() {
        return Err(Error::NothingToSave)
    } else {
        None
    };
    Ok(path)
}

fn get_puzzles_iter<'a>(context: &'a Context) -> impl Iterator<Item=Result<Puzzle>> + 'a {
    let generator = move || {
        match &context.source {
            SourceContext::File(path) => {
                yield read_puzzle_file(Path::new(path));
            },
            &SourceContext::Generate(GenerateContext { width, .. }) => {
                loop {
                    println!("Generating puzzle");
                    yield Ok(puzzle::generate_puzzle(width))
                }
            },
        }
    };
    gen_to_iter(generator)
}

fn read_puzzle_file(path: &Path) -> Result<Puzzle> {
    println!("reading puzzle from \"{}\"", path.display());
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let puzzle = puzzle::parse(&buf).map_err(Error::ParsePuzzle)?;
    Ok(puzzle)
}

fn puzzle_path_iterator(root: &Path) -> impl Iterator<Item=PathBuf> {
    let root = root.to_path_buf();
    (1..)
        .map(move |i| {
            let mut path = root.clone();
            path.push(format!("puzzle_{}", i));
            path
        })
        .filter(|path| !path.exists())
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

fn solve_puzzle(puzzle: &Puzzle, step_images_path: Option<&Path>, image_width: Option<u32>) -> std::result::Result<PuzzleMarkup, SolveError> {
    println!("Solving puzzle");
    let mut solver = PuzzleSolver::new(puzzle);
    if let Some(path) = step_images_path {
        solver.save_steps(path);
    }
    if let Some(image_width) = image_width {
        solver.steps_image_width(image_width);
    }
    Ok(solver.solve()?)
}

fn build_context<'a>(matches: &'a clap::ArgMatches) -> Context<'a> {
    Context {
        image_width: matches.value_of("image_width")
            .map(|s| s.parse().expect("invalid image width")),
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
                unsolvable: matches.is_present("unsolvable"),
            }),
        },
        solve: if matches.is_present("solve") {
            Some(SolveContext {
                save_image: matches.is_present("save_solved_image"),
                save_step_images: matches.is_present("save_step_images"),
            })
        } else { None },
        save_image: matches.is_present("save_image"),
    }
}

struct PuzzleHandler {
    image_width: Option<u32>,
    save_any: bool,
    save_puzzle: bool,
    save_image: bool,
    solve_handler: Option<SolvePuzzleHandler>,
}

impl PuzzleHandler {
    fn new(context: &Context) -> Self {
        let save_puzzle: bool;
        let skip_unsolved: bool;
        match &context.source {
            SourceContext::Generate(context) => {
                save_puzzle = context.save_puzzle;
                skip_unsolved = !context.unsolvable;
            },
            _ => {
                save_puzzle = false;
                skip_unsolved = false;
            },
        }
        let image_width = context.image_width;
        let solve_handler = context.solve.as_ref()
            .map(|context| SolvePuzzleHandler::new(context, skip_unsolved, image_width));
        Self {
            image_width,
            save_any: context.save_any(),
            save_puzzle,
            save_image: context.save_image,
            solve_handler,
        }
    }

    fn handle_puzzle(&mut self, puzzle: &Puzzle) -> Result<Option<PuzzleTempDir>> {
        print_puzzle(puzzle);
        let dir = if self.save_any {
            Some(PuzzleTempDir::new()?)
        } else {
            None
        };
        if let Some(solve_handler) = &self.solve_handler {
            let solved = solve_handler.handle_puzzle(puzzle, dir.as_ref())?;
            if !solved {
                return Ok(None)
            }
        };
        if self.save_puzzle {
            dir.as_ref().unwrap().write_puzzle(puzzle)?;
        }
        if self.save_image {
            let mut builder = PuzzleImageBuilder::new(puzzle);
            if let Some(image_width) = self.image_width {
                builder.image_width(image_width);
            }
            let image = builder.build();
            dir.as_ref().unwrap().write_puzzle_image(image)?;
        }
        Ok(dir)
    }
}

struct SolvePuzzleHandler {
    image_width: Option<u32>,
    save_image: bool,
    save_step_images: bool,
    skip_unsolved: bool,
}

impl SolvePuzzleHandler {
    fn new(context: &SolveContext, skip_unsolved: bool, image_width: Option<u32>) -> Self {
        let &SolveContext { save_image, save_step_images } = context;
        Self {
            image_width,
            save_image,
            save_step_images,
            skip_unsolved,
        }
    }

    fn handle_puzzle(&self, puzzle: &Puzzle, dir: Option<&PuzzleTempDir>) -> Result<bool> {
        let markup = {
            let step_images_path = if self.save_step_images {
                let path = dir.unwrap().steps_path();
                fs::create_dir(&path)?;
                Some(path)
            } else { None };
            solve_puzzle(puzzle, step_images_path.as_ref().map(PathBuf::as_path), self.image_width)
                .map_err(Error::Solve)?
        };

        let solved = markup.is_solved();

        if solved {
            println!("Puzzle solved");
        } else {
            println!("Puzzle could not be solved");
        }

        if self.skip_unsolved && !solved {
            return Ok(false)
        }

        if self.save_image {
            let mut builder = PuzzleImageBuilder::new(puzzle);
            builder.cell_variables(Some(&markup.cell_variables));
            if let Some(image_width) = self.image_width {
                builder.image_width(image_width);
            }
            let image = builder.build();
            dir.unwrap().write_saved_puzzle_image(image)?;
        }

        Ok(solved)
    }
}


struct Context<'a> {
    image_width: Option<u32>,
    output_path: Option<&'a str>,
    source: SourceContext<'a>,
    solve: Option<SolveContext>,
    save_image: bool,
}

impl<'a> Context<'a> {

    /// returns true if any files are to be saved
    fn save_any(&self) -> bool {
        if self.save_image {
            return true
        }
        if let SourceContext::Generate(ref gc) = &self.source {
            if gc.save_puzzle {
                return true
            }
        }
        if let Some(ref sc) = &self.solve {
            if sc.save_image || sc.save_step_images {
                return true
            }
        }
        false
    }
}

enum SourceContext<'a> {
    File(&'a str),
    Generate(GenerateContext),
}

struct GenerateContext {
    count: u32,
    width: u32,
    save_puzzle: bool,
    unsolvable: bool,
}

struct SolveContext {
    save_image: bool,
    save_step_images: bool,
}

enum Error {
    CreateDirectory(PathBuf, io::Error),
    Io(io::Error),
    NothingToSave,
    ParsePuzzle(String),
    Solve(SolveError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CreateDirectory(path, e) => write!(f, "Error creating directory {}: {}", path.display(), e),
            Error::Io(e) => write!(f, "{}", e),
            Error::NothingToSave => write!(f, "output path specified but nothing to save"),
            Error::ParsePuzzle(e) => write!(f, "Error parsing puzzle: {}", e),
            Error::Solve(e) => write!(f, "Error solving puzzle: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
