#![feature(generators, generator_trait, type_alias_impl_trait)]

extern crate camcam;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate clap;
extern crate tempfile;
extern crate image;

mod cli_error;
mod context;
mod options;
mod puzzle_folder_builder;

use crate::puzzle_folder_builder::PuzzleFolderBuilder;
use camcam::puzzle::Puzzle;
use camcam::puzzle;
use camcam::puzzle::solve::PuzzleSolver;
use camcam::puzzle::solve::PuzzleMarkup;
use log::LogLevel;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use camcam::puzzle::PuzzleImageBuilder;
use crate::options::Options;
use crate::context::Context;
use crate::cli_error::CliError;
use camcam::puzzle::error::SolveError;

type Result<T> = std::result::Result<T, CliError>;

const DEFAULT_PUZZLE_WIDTH: u32 = 4;
const DEFAULT_PATH: &str = "output";

fn main() -> Result<()> {
    env_logger::init().unwrap();

    let options = Options::from_args();
    let mut context = Context::new(options)?;
    let puzzles = puzzles_iter(context.options());
    for puzzle in puzzles {
        let puzzle = puzzle?;
        handle_puzzle(&mut context, &puzzle)?;
    }
    Ok(())
}

fn puzzles_iter(options: &Options) -> Box<dyn Iterator<Item=Result<Puzzle>>> {
    match &options.source {
        options::Source::File(path) => {
            let path = PathBuf::from(path);
            Box::new(std::iter::once_with(move || {
                println!("reading puzzle from \"{}\"", path.display());
                let puzzle = Puzzle::from_file(&path)?;
                Ok(puzzle)
            }))
        }
        &options::Source::Generate(options::Generate { count, width, .. }) => {
            Box::new((0..count).map(move |_| {
                println!("Generating puzzle");
                Ok(Puzzle::generate(width))
            }))
        }
    }
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

fn handle_puzzle(context: &mut Context, puzzle: &Puzzle) -> Result<()> {
    if context.options().save_any() {
        context.set_temp_dir(PuzzleFolderBuilder::new()?)
    }
    print_puzzle(puzzle);
    if context.options().solve.is_some() {
        let solved = solve_handle_puzzle(context, puzzle)?;
        if !solved {
            return Ok(());
        }
    };
    if let options::Source::Generate(options::Generate { save_puzzle: true, .. }) = &context.options().source {
            context.folder_builder().unwrap().write_puzzle(puzzle)?;
    }
    if context.options().save_image {
        save_image(context, puzzle)?;
    }
    if context.folder_builder().is_some() {
        let target_path = context.next_path();
        context.folder_builder().unwrap().save(&target_path)?;
        println!("Saved puzzle to {}", target_path.display());
    }
    Ok(())
}

fn save_image(context: &Context, puzzle: &Puzzle) -> Result<()> {
    let mut builder = PuzzleImageBuilder::new(puzzle);
    if let Some(image_width) = context.options().image_width {
        builder.image_width(image_width);
    }
    let image = builder.build();
    context.folder_builder().unwrap().write_puzzle_image(image)?;
    Ok(())
}

fn solve_handle_puzzle(context: &Context, puzzle: &Puzzle) -> Result<bool> {
    let image_width = context.options().image_width;
    let solve_context = context.options().solve.as_ref().unwrap();
    let markup = {
        let step_images_path = if solve_context.save_step_images {
            let path = context.folder_builder().unwrap().steps_path();
            fs::create_dir(&path)?;
            Some(path)
        } else { None };
        solve_puzzle(puzzle, step_images_path.as_ref().map(PathBuf::as_path), image_width)?
    };

    let solved = markup.is_solved();

    if solved {
        println!("Puzzle solved");
    } else {
        println!("Puzzle could not be solved");
    }

    if let options::Source::Generate(context) = &context.options().source {
        if !context.unsolvable && !solved {
            return Ok(false);
        }
    }

    if solve_context.save_image {
        let mut builder = PuzzleImageBuilder::new(puzzle);
        builder.cell_variables(Some(&markup.cell_variables));
        if let Some(image_width) = image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        context.folder_builder().unwrap().write_saved_puzzle_image(image)?;
    }

    Ok(solved)
}
