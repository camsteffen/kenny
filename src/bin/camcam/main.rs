#[deny(trivial_numeric_casts)]
#[deny(trivial_casts)]
extern crate camcam;
extern crate clap;
extern crate env_logger;
extern crate image;
#[macro_use]
extern crate log;
extern crate tempfile;

use std::fs;
use std::path::PathBuf;

use failure::Fallible;

use camcam::puzzle::Puzzle;
use camcam::puzzle::PuzzleImageBuilder;
use camcam::puzzle::solve::{PuzzleSolver, SolveResult};

use crate::context::{Context, PuzzleContext};
use crate::options::Options;
use itertools::Itertools;
use std::panic::{catch_unwind, resume_unwind};

mod context;
mod options;
mod puzzle_folder_builder;

fn main() -> Fallible<()> {
    env_logger::init();
    let options = Options::from_args()?;
    let context = Context::new(options)?;
    start(&context)?;
    Ok(())
}

fn start(context: &Context) -> Fallible<()> {
    match context.options().source() {
        options::Source::File(path) => {
            start_file(context, path)?;
        }
        &options::Source::Generate(options::Generate { count, width, .. }) => {
            start_generate(context, count, width)?;
        },
    }
    Ok(())
}

fn start_file(context: &Context, path: &str) -> Fallible<()> {
    let path: PathBuf = path.into();
    println!("reading puzzle from \"{}\"", path.display());
    let puzzle = Puzzle::from_file(path);
    on_puzzle_sourced(context, puzzle?)?;
    Ok(())
}

fn start_generate(context: &Context, count: u32, width: usize) -> Fallible<()> {
    let mut included_count = 0;
    while included_count < count {
        println!("Generating puzzle {}/{}", included_count + 1, count);
        let puzzle = Puzzle::generate_untested(width);
        let included = on_puzzle_sourced(context, puzzle)?;
        if included {
            included_count += 1;
        }
    }
    Ok(())
}

fn on_puzzle_sourced(context: &Context, puzzle: Puzzle) -> Fallible<bool> {
    let mut puzzle_context = PuzzleContext::new(context.options(), puzzle)?;
    log_puzzle(puzzle_context.puzzle());
    on_save_puzzle(&puzzle_context)?;
    let result = catch_unwind(|| {
        on_solve_puzzle(&puzzle_context)
    });
    // in case of a panic or error, save the puzzle output
    let should_save = result.as_ref().map_or(false, |included| included.as_ref().ok().copied().unwrap_or(false));
    if should_save {
        puzzle_context.save_folder_if_present(context)
            .unwrap_or_else(|e| error!("Error saving puzzle: {}", e));
    }
    match result {
        Ok(included) => Ok(included?),
        Err(cause) => resume_unwind(cause),
    }
}

fn log_puzzle(puzzle: &Puzzle) {
    if log_enabled!(log::Level::Info) {
        let cages = puzzle.cages().enumerate()
            .map(|(i, cage)| format!(" {:>2}: {} {}", i, &cage.operator().symbol().unwrap_or(' '), cage.target()))
            .join("\n");
        info!("Cell Cage IDs:\n{}Cages:\n{}", puzzle.cell_cage_indices(), cages);
    }
}

fn on_solve_puzzle(context: &PuzzleContext) -> Fallible<bool> {
    if let Some(solve) = context.options().solve() {
        let solved = on_do_solve_puzzle(&context, solve)?;
        let include = context.options().source().generate().map_or(true, |generate|
            if solved { generate.include_solvable } else { generate.include_unsolvable });
        Ok(include)
    } else {
        Ok(true)
    }
}

fn on_save_puzzle(context: &PuzzleContext) -> Fallible<()> {
    if context.options().source().generate().map_or(false, |g| g.save_puzzle) {
        context.folder_builder().unwrap().write_puzzle(context.puzzle())?;
    }
    on_save_image(context)?;
    Ok(())
}

fn on_save_image(context: &PuzzleContext) -> Fallible<()> {
    if context.options().save_image() {
        save_image(&context)?;
    }
    Ok(())
}

fn save_image(puzzle_context: &PuzzleContext) -> Fallible<()> {
    let mut builder = PuzzleImageBuilder::new(puzzle_context.puzzle());
    if let Some(image_width) = puzzle_context.options().image_width {
        builder.image_width(image_width);
    }
    let image = builder.build();
    puzzle_context.folder_builder().unwrap().write_puzzle_image(image)?;
    Ok(())
}

fn on_do_solve_puzzle(context: &PuzzleContext, solve_options: &options::Solve) -> Fallible<bool> {
    println!("Solving puzzle");
    let solver = build_solver(context, solve_options)?;
    let solution = match solver.solve()? {
        SolveResult::Unsolvable => {
            println!("Puzzle is not solvable");
            None
        },
        SolveResult::Solved(solution) => {
            println!("Puzzle solved");
            Some(solution)
        },
        SolveResult::MultipleSolutions => {
            println!("Puzzle has multiple solutions");
            None
        },
    };

    if let options::Source::Generate(context) = context.options().source() {
        if !context.include_unsolvable && !solution.is_some() {
            return Ok(false);
        }
    }

    if solve_options.save_image {
        let mut builder = PuzzleImageBuilder::new(context.puzzle());
        builder.solution(solution.as_ref());
        if let Some(image_width) = context.options().image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        context.folder_builder().unwrap().write_solved_puzzle_image(image)?;
    }

    Ok(solution.is_some())
}

// todo refactor
fn build_solver<'a>(context: &'a PuzzleContext, solve_options: &options::Solve) -> Fallible<PuzzleSolver<'a>> {
    let mut solver = PuzzleSolver::new(context.puzzle());
    if solve_options.save_step_images {
        let path = context.folder_builder().unwrap().steps_path();
        fs::create_dir(&path)?;
        solver.save_steps(&path);
    }
    if let Some(image_width) = context.options().image_width {
        solver.steps_image_width(image_width);
    }
    Ok(solver)
}
