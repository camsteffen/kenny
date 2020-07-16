#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]

#[macro_use]
extern crate log;

use std::fs;
use std::panic::{catch_unwind, resume_unwind};

use anyhow::Result;
use camcam::puzzle::image::PuzzleImageBuilder;
use camcam::puzzle::solve::{PuzzleSolver, SolveResult};
use camcam::puzzle::{Puzzle, Solution};
use itertools::Itertools;

use crate::context::{Context, PuzzleContext};
use crate::options::Options;

mod context;
mod options;
mod puzzle_folder_builder;

fn main() -> Result<()> {
    env_logger::init();
    let options = Options::from_args()?;
    let mut context = Context::new(options)?;
    context.start()?;
    Ok(())
}

impl Context {
    fn start(&mut self) -> Result<()> {
        match self.options().source() {
            options::Source::File(_) => {
                self.start_file()?;
            }
            &options::Source::Generate(options::Generate { count, width, .. }) => {
                self.start_generate(count, width)?;
            }
        }
        Ok(())
    }

    fn start_file(&mut self) -> Result<()> {
        let path = self.options().source().file().unwrap();
        println!("Reading puzzle from \"{}\"", path.display());
        let puzzle = Puzzle::from_file(path)?;
        let mut context = PuzzleContext::new(self, &puzzle)?;
        context.on_puzzle_sourced()?;
        Ok(())
    }

    fn start_generate(&mut self, count: u32, width: usize) -> Result<()> {
        let mut included_count = 0;
        let mut attempt = 1;
        while included_count < count {
            println!(
                "Generating puzzle {}/{}{attempt}",
                included_count + 1,
                count,
                attempt = if attempt == 1 {
                    String::new()
                } else {
                    format!(" (attempt {})", attempt)
                }
            );
            let puzzle = Puzzle::generate_untested(width);
            let mut context = PuzzleContext::new(self, &puzzle)?;
            let included = context.on_puzzle_sourced()?;
            if included {
                included_count += 1;
                attempt = 1;
            } else {
                println!("Puzzle discarded");
                attempt += 1;
            }
        }
        Ok(())
    }
}

impl PuzzleContext<'_> {
    fn on_puzzle_sourced(&mut self) -> Result<bool> {
        log_puzzle(self.puzzle());
        self.save_puzzle()?;
        let solve_options = match self.options().solve() {
            None => return Ok(true),
            Some(solve_options) => solve_options,
        };

        // catch a panic to save puzzle output
        let result = catch_unwind(|| self.on_solve_puzzle(solve_options));
        let include = match result {
            Ok(Ok(ref result)) => self.should_include(result),
            // if any errors occurred, save output anyways
            _ => true,
        };
        if include {
            match self.save_folder_if_present() {
                // propagate the error only if there are no other errors
                Err(e) if matches!(result, Ok(Ok(_))) => return Err(e),
                _ => (),
            }
        }
        match result {
            Err(e) => resume_unwind(e),
            Ok(result) => result?,
        };
        Ok(include)
    }

    fn should_include(&self, result: &SolveResult) -> bool {
        let context = match self.options().source() {
            options::Source::Generate(context) => context,
            _ => return true,
        };
        if let Some(solve) = result.solved() {
            context.include_solvable
                && (!context.require_search || solve.used_search)
                && (!context.no_require_search || !solve.used_search)
        } else {
            context.include_unsolvable
        }
    }

    fn save_puzzle(&self) -> Result<()> {
        if self
            .options()
            .source()
            .generate()
            .map_or(false, |g| g.save_puzzle)
        {
            self.folder_builder().unwrap().write_puzzle(self.puzzle())?;
        }
        if self.options().save_image() {
            self.save_image()?;
        }
        Ok(())
    }

    fn save_image(&self) -> Result<()> {
        let image = PuzzleImageBuilder::new(self.puzzle()).build();
        self.folder_builder().unwrap().write_puzzle_image(&image)?;
        Ok(())
    }

    fn on_solve_puzzle(&self, solve_options: &options::Solve) -> Result<SolveResult> {
        let solver = self.build_solver(solve_options)?;
        let result = solver.solve()?;
        let msg = match result {
            SolveResult::Unsolvable => "Puzzle is not solvable",
            SolveResult::Solved(_) => "Puzzle solved",
            SolveResult::MultipleSolutions => "Puzzle has multiple solutions",
        };
        println!("{}", msg);
        if self.should_include(&result) {
            if let Some(result) = result.solved() {
                self.save_solved_image(solve_options, &result.solution)?;
            }
        }
        Ok(result)
    }

    fn save_solved_image(&self, solve_options: &options::Solve, solution: &Solution) -> Result<()> {
        if solve_options.save_image {
            let mut builder = PuzzleImageBuilder::new(self.puzzle());
            builder.solution(solution);
            let image = builder.build();
            self.folder_builder()
                .unwrap()
                .write_solved_puzzle_image(&image)?;
        }
        Ok(())
    }

    fn build_solver(&self, solve_options: &options::Solve) -> Result<PuzzleSolver<'_>> {
        let mut solver = PuzzleSolver::new(self.puzzle());
        if solve_options.save_step_images {
            let path = self.folder_builder().unwrap().steps_path();
            fs::create_dir(&path)?;
            solver.save_steps(&path);
        }
        Ok(solver)
    }

    pub fn save_folder_if_present(&mut self) -> Result<()> {
        let folder_builder = match self.take_folder_builder() {
            None => return Ok(()),
            Some(folder_builder) => folder_builder,
        };
        let path = self.next_puzzle_path();
        folder_builder.save(&path)?;
        println!("Saved puzzle to {}", path.display());
        Ok(())
    }
}

fn log_puzzle(puzzle: &Puzzle) {
    if log_enabled!(log::Level::Info) {
        let cages = puzzle
            .cages()
            .enumerate()
            .map(|(i, cage)| {
                format!(
                    " {:>2}: {} {}",
                    i,
                    &cage.operator().symbol().unwrap_or(' '),
                    cage.target()
                )
            })
            .join("\n");
        info!(
            "Cell Cage IDs:\n{}Cages:\n{}",
            puzzle.cell_cage_indices(),
            cages
        );
    }
}
