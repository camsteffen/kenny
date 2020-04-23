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
use camcam::puzzle::solve::PuzzleSolver;

use crate::context::{Context, PuzzleContext};
use crate::options::Options;
use std::iter;

mod context;
mod options;
mod puzzle_folder_builder;

const DEFAULT_PUZZLE_WIDTH: usize = 4;
const DEFAULT_PATH: &str = "output";

fn main() -> Fallible<()> {
    env_logger::init();

    let options = Options::from_args()?;
    let mut context = Context::new(options)?;
    let puzzles = puzzles_iter(context.options());
    for puzzle in puzzles {
        on_puzzle_sourced(&mut context, puzzle?)?;
    }
    Ok(())
}

fn puzzles_iter(options: &Options) -> Box<dyn Iterator<Item=Fallible<Puzzle>>> {
    match options.source() {
        options::Source::File(path) => {
            let path = PathBuf::from(path);
            Box::new(iter::once_with(move || {
                println!("reading puzzle from \"{}\"", path.display());
                Puzzle::from_file(path)
            }))
        }
        &options::Source::Generate(options::Generate { count, width, .. }) => {
            Box::new((0..count).map(move |i| {
                println!("Generating puzzle {}/{}", i + 1, count);
                Ok(Puzzle::generate_untested(width))
            }))
        }
    }
}

fn on_puzzle_sourced(context: &mut Context, puzzle: Puzzle) -> Fallible<()> {
    let mut puzzle_context = PuzzleContext::new(context.options().clone(), puzzle)?;
    log_puzzle(puzzle_context.puzzle());
    if !on_solve_puzzle(&puzzle_context)? {
        return Ok(())
    }
    on_save_puzzle(&puzzle_context)?;
    on_save_image(&puzzle_context)?;
    puzzle_context.save_folder_if_present(context)?;
    Ok(())
}

fn log_puzzle(puzzle: &Puzzle) {
    if log_enabled!(log::Level::Info) {
        info!("Cell Cage Indices:\n{}", puzzle.cell_cage_indices());
        info!("Cages:");
        for (i, cage) in puzzle.cages().iter().enumerate() {
            info!(" {:>2}: {} {}", i, &cage.operator().symbol().unwrap_or(' '), cage.target());
        }
    }
}

fn on_solve_puzzle(context: &PuzzleContext) -> Fallible<bool> {
    if let Some(solve) = context.options().solve() {
        let solved = on_do_solve_puzzle(&context, solve)?;
        if !solved && !context.options().source().generate().map_or(false, |g| g.include_unsolvable) {
            return Ok(false);
        }
    };
    Ok(true)
}

fn on_save_puzzle(context: &PuzzleContext) -> Fallible<()> {
    if context.options().source().generate().map_or(false, |g| g.save_puzzle) {
        context.folder_builder().unwrap().write_puzzle(context.puzzle())?;
    }
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
    let solver = build_solver(context)?;
    let markup = solver.solve()?;
    let solved = if let Some(solution) = markup.solution() {
        assert!(context.puzzle().verify_solution(&solution));
        println!("Puzzle solved");
        true
    } else {
        println!("Puzzle could not be solved");
        false
    };

    if let options::Source::Generate(context) = context.options().source() {
        if !context.include_unsolvable && !solved {
            return Ok(false);
        }
    }

    if solve_options.save_image {
        let mut builder = PuzzleImageBuilder::new(context.puzzle());
        builder.cell_variables(Some(&markup.cells()));
        if let Some(image_width) = context.options().image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        context.folder_builder().unwrap().write_solved_puzzle_image(image)?;
    }

    Ok(solved)
}

fn build_solver(context: &PuzzleContext) -> Fallible<PuzzleSolver> {
    let mut solver = PuzzleSolver::new(context.puzzle());
    if context.options().solve().unwrap().save_step_images {
        let path = context.folder_builder().unwrap().steps_path();
        fs::create_dir(&path)?;
        solver.save_steps(&path);
    }
    if let Some(image_width) = context.options().image_width {
        solver.steps_image_width(image_width);
    }
    Ok(solver)
}
