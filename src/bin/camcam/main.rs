#![warn(rust_2018_idioms)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]

#[macro_use]
extern crate log;

use std::fs;
use std::panic::{catch_unwind, resume_unwind};

use failure::Fallible;
use itertools::Itertools;

use camcam::puzzle::solve::{PuzzleSolver, SolveResult};
use camcam::puzzle::PuzzleImageBuilder;
use camcam::puzzle::{Puzzle, Solution};

use crate::context::{Context, PuzzleContext};
use crate::options::Options;

mod context;
mod options;
mod puzzle_folder_builder;

fn main() -> Fallible<()> {
    env_logger::init();
    let options = Options::from_args()?;
    let mut context = Context::new(options)?;
    context.start()?;
    Ok(())
}

impl Context {
    fn start(&mut self) -> Fallible<()> {
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

    fn start_file(&mut self) -> Fallible<()> {
        let path = self.options().source().file().unwrap();
        println!("reading puzzle from \"{}\"", path.display());
        let puzzle = Puzzle::from_file(path)?;
        let mut context = PuzzleContext::new(self, &puzzle)?;
        context.on_puzzle_sourced()?;
        Ok(())
    }

    fn start_generate(&mut self, count: u32, width: usize) -> Fallible<()> {
        let mut included_count = 0;
        while included_count < count {
            println!("Generating puzzle {}/{}", included_count + 1, count);
            let puzzle = Puzzle::generate_untested(width);
            let mut context = PuzzleContext::new(self, &puzzle)?;
            let included = context.on_puzzle_sourced()?;
            if included {
                included_count += 1;
            }
        }
        Ok(())
    }
}

impl PuzzleContext<'_> {
    fn on_puzzle_sourced(&mut self) -> Fallible<bool> {
        // let mut puzzle_context = PuzzleContext::new(context.options(), puzzle)?;
        log_puzzle(self.puzzle());
        self.save_puzzle()?;
        let result = catch_unwind(|| self.on_solve_puzzle());
        // in case of a panic or error, save the puzzle output
        let should_save = result.as_ref().map_or(false, |included| {
            included.as_ref().ok().copied().unwrap_or(false)
        });
        if should_save {
            self.save_folder_if_present()
                .unwrap_or_else(|e| error!("Error saving puzzle: {}", e));
        }
        match result {
            Ok(included) => Ok(included?),
            Err(cause) => resume_unwind(cause),
        }
    }

    fn on_solve_puzzle(&self) -> Fallible<bool> {
        if let Some(solve) = self.options().solve() {
            let solved = self.on_do_solve_puzzle(solve)?;
            let include = self.options().source().generate().map_or(true, |generate| {
                if solved {
                    generate.include_solvable
                } else {
                    generate.include_unsolvable
                }
            });
            Ok(include)
        } else {
            Ok(true)
        }
    }

    fn save_puzzle(&self) -> Fallible<()> {
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

    fn save_image(&self) -> Fallible<()> {
        let mut builder = PuzzleImageBuilder::new(self.puzzle());
        if let Some(image_width) = self.options().image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        self.folder_builder().unwrap().write_puzzle_image(image)?;
        Ok(())
    }

    fn on_do_solve_puzzle(&self, solve_options: &options::Solve) -> Fallible<bool> {
        println!("Solving puzzle");
        let solver = self.build_solver(solve_options)?;
        let solution = match solver.solve()? {
            SolveResult::Unsolvable => {
                println!("Puzzle is not solvable");
                None
            }
            SolveResult::Solved(solution) => {
                println!("Puzzle solved");
                Some(solution)
            }
            SolveResult::MultipleSolutions => {
                println!("Puzzle has multiple solutions");
                None
            }
        };

        if let options::Source::Generate(context) = self.options().source() {
            if !context.include_unsolvable && !solution.is_some() {
                return Ok(false);
            }
        }

        self.save_solved_image(solve_options, &solution)?;

        Ok(solution.is_some())
    }

    fn save_solved_image(
        &self,
        solve_options: &options::Solve,
        solution: &Option<Solution>,
    ) -> Fallible<()> {
        if solve_options.save_image {
            let mut builder = PuzzleImageBuilder::new(self.puzzle());
            builder.solution(solution.as_ref());
            if let Some(image_width) = self.options().image_width {
                builder.image_width(image_width);
            }
            let image = builder.build();
            self.folder_builder()
                .unwrap()
                .write_solved_puzzle_image(image)?;
        }
        Ok(())
    }

    fn build_solver(&self, solve_options: &options::Solve) -> Fallible<PuzzleSolver<'_>> {
        let mut solver = PuzzleSolver::new(self.puzzle());
        if solve_options.save_step_images {
            let path = self.folder_builder().unwrap().steps_path();
            fs::create_dir(&path)?;
            solver.save_steps(&path);
        }
        if let Some(image_width) = self.options().image_width {
            solver.steps_image_width(image_width);
        }
        Ok(solver)
    }

    pub fn save_folder_if_present(&mut self) -> Fallible<()> {
        if let Some(folder_builder) = self.take_folder_builder() {
            let path = self.next_puzzle_path();
            folder_builder.save(&path)?;
            println!("Saved puzzle to {}", path.display());
        }
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
