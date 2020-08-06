#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]

use std::fs;
use std::io::stdout;
use std::panic::{catch_unwind, resume_unwind};

use anyhow::Result;
use kenny::image::PuzzleImageBuilder;
use kenny::puzzle::{Puzzle, Solution};
use kenny::solve::{PuzzleSolver, SolveResult};

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
        print_puzzle::print_puzzle(self.puzzle(), &mut stdout()).unwrap();
        self.save_puzzle()?;
        let unwind_result = self.options().solve().map(|solve_options| {
            // catch a panic to save puzzle output
            catch_unwind(|| self.on_solve_puzzle(solve_options))
        });
        let save_folder = match unwind_result {
            Some(Ok(Ok(ref result))) => self.should_include(result),
            None | Some(Err(_)) | Some(Ok(Err(_))) => true,
        };
        let save_result = if save_folder {
            Some(self.save_folder_if_present())
        } else {
            None
        };
        if let Some(unwind_result) = unwind_result {
            match unwind_result {
                Err(e) => resume_unwind(e),
                Ok(solve_result) => {
                    solve_result?;
                }
            }
        }
        if let Some(result) = save_result {
            // propagate save error after checking for other errors
            result?;
        }
        Ok(save_folder)
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
        if self.options().save_puzzle() {
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

mod print_puzzle {
    use std::io::Write;

    use ahash::AHashMap;
    use once_cell::sync::Lazy;

    use kenny::puzzle::Puzzle;
    use kenny::square::IsSquare;
    use kenny::Coord;

    pub fn print_puzzle(puzzle: &Puzzle, w: &mut impl Write) -> std::io::Result<()> {
        const CELL_INNER_WIDTH: usize = 4;

        let width = puzzle.width() * 2 + 1;
        // iterate each position where there is a position for every cell, cell border, and corner
        for i in 0..width.pow(2) {
            if i & 1 == 0 && i % width & 1 == 0 {
                // corner
                let top_edge = i < width;
                let bottom_edge = i / width == width - 1;
                let left_edge = i % width == 0;
                let right_edge = i % width == width - 1;
                let cell_right = i % width / 2;
                let cell_left = cell_right.wrapping_sub(1);
                let cell_down = i / width / 2;
                let cell_up = cell_down.wrapping_sub(1);
                let left_up = Coord::new(cell_left, cell_up);
                let right_up = Coord::new(cell_right, cell_up);
                let left_down = Coord::new(cell_left, cell_down);
                let right_down = Coord::new(cell_right, cell_down);
                let up = if top_edge {
                    0
                } else if left_edge || right_edge || puzzle.is_cage_border(left_up, right_up) {
                    UP_HEAVY
                } else {
                    UP
                };
                let down = if bottom_edge {
                    0
                } else if left_edge || right_edge || puzzle.is_cage_border(left_down, right_down) {
                    DOWN_HEAVY
                } else {
                    DOWN
                };
                let left = if left_edge {
                    0
                } else if top_edge || bottom_edge || puzzle.is_cage_border(left_up, left_down) {
                    LEFT_HEAVY
                } else {
                    LEFT
                };
                let right = if right_edge {
                    0
                } else if top_edge || bottom_edge || puzzle.is_cage_border(right_up, right_down) {
                    RIGHT_HEAVY
                } else {
                    RIGHT
                };
                write!(w, "{}", box_char(up | down | left | right))?;
            } else if i & 1 == 1 {
                // edge
                if i / width & 1 == 0 {
                    // horizontal edge
                    let col = i % width / 2;
                    let row_down = i / width / 2;
                    let row_up = row_down.wrapping_sub(1);
                    let up = Coord::new(col, row_up);
                    let down = Coord::new(col, row_down);
                    let v =
                        if i < width || i / width == width - 1 || puzzle.is_cage_border(up, down) {
                            LEFT_HEAVY | RIGHT_HEAVY
                        } else {
                            LEFT | RIGHT
                        };
                    write!(w, "{}", box_char(v).to_string().repeat(CELL_INNER_WIDTH))?;
                } else {
                    // vertical edge
                    let row = i / width / 2;
                    let col_right = i % width / 2;
                    let col_left = col_right.wrapping_sub(1);
                    let left = Coord::new(col_left, row);
                    let right = Coord::new(col_right, row);
                    let v = if i % width == 0
                        || i % width == width - 1
                        || puzzle.is_cage_border(left, right)
                    {
                        UP_HEAVY | DOWN_HEAVY
                    } else {
                        UP | DOWN
                    };
                    write!(w, "{}", box_char(v))?;
                };
            } else {
                // cell
                let col = i % width / 2;
                let row = i / width / 2;
                let cell = puzzle.cell(Coord::new(col, row));
                let cage = cell.cage();
                if cage.cell_ids()[0] == cell.id() {
                    // cell with cage markup
                    write!(
                        w,
                        "{}{:<3}",
                        cage.operator().display_symbol().unwrap_or(' '),
                        cage.target()
                    )?;
                } else {
                    // blank cell
                    write!(w, "{}", " ".repeat(CELL_INNER_WIDTH))?;
                }
            }
            if i % width == width - 1 {
                writeln!(w)?;
            }
        }
        Ok(())
    }

    const UP: u8 = 1 << 0;
    const DOWN: u8 = 1 << 1;
    const LEFT: u8 = 1 << 2;
    const RIGHT: u8 = 1 << 3;
    const UP_HEAVY: u8 = 1 << 4;
    const DOWN_HEAVY: u8 = 1 << 5;
    const LEFT_HEAVY: u8 = 1 << 6;
    const RIGHT_HEAVY: u8 = 1 << 7;

    // only contains needed characters
    const BOX_CHAR_MAP: Lazy<AHashMap<u8, char>> = Lazy::new(|| {
        let mut map = AHashMap::new();
        // use space instead of light lines to make cages easier to see
        map.insert(UP | DOWN, ' ');
        map.insert(LEFT | RIGHT, ' ');

        // cage edges
        map.insert(UP_HEAVY | DOWN_HEAVY, '┃');
        map.insert(LEFT_HEAVY | RIGHT_HEAVY, '━');

        // corners
        map.insert(UP_HEAVY | RIGHT_HEAVY, '┗');
        map.insert(DOWN_HEAVY | RIGHT_HEAVY, '┏');
        map.insert(DOWN_HEAVY | LEFT_HEAVY, '┓');
        map.insert(UP_HEAVY | LEFT_HEAVY, '┛');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT, '┨');
        map.insert(UP_HEAVY | DOWN_HEAVY | RIGHT, '┠');
        map.insert(UP | LEFT_HEAVY | RIGHT_HEAVY, '┷');
        map.insert(DOWN | LEFT_HEAVY | RIGHT_HEAVY, '┯');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT_HEAVY, '┫');
        map.insert(UP_HEAVY | DOWN_HEAVY | RIGHT_HEAVY, '┣');
        map.insert(UP_HEAVY | LEFT_HEAVY | RIGHT_HEAVY, '┻');
        map.insert(DOWN_HEAVY | LEFT_HEAVY | RIGHT_HEAVY, '┳');
        map.insert(UP | DOWN | LEFT | RIGHT, '┼');
        map.insert(UP_HEAVY | DOWN | LEFT | RIGHT, '╀');
        map.insert(UP | DOWN_HEAVY | LEFT | RIGHT, '╁');
        map.insert(UP | DOWN | LEFT_HEAVY | RIGHT_HEAVY, '┿');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT | RIGHT, '╂');
        map.insert(UP | DOWN_HEAVY | LEFT | RIGHT_HEAVY, '╆');
        map.insert(UP | DOWN_HEAVY | LEFT_HEAVY | RIGHT, '╅');
        map.insert(UP_HEAVY | DOWN | LEFT_HEAVY | RIGHT, '╃');
        map.insert(UP_HEAVY | DOWN | LEFT | RIGHT_HEAVY, '╄');
        map.insert(UP | DOWN_HEAVY | LEFT_HEAVY | RIGHT_HEAVY, '╈');
        map.insert(UP_HEAVY | DOWN | LEFT_HEAVY | RIGHT_HEAVY, '╇');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT | RIGHT_HEAVY, '╊');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT_HEAVY | RIGHT, '╉');
        map.insert(UP_HEAVY | DOWN_HEAVY | LEFT_HEAVY | RIGHT_HEAVY, '╋');
        map
    });

    pub fn box_char(id: u8) -> char {
        *BOX_CHAR_MAP
            .get(&id)
            .unwrap_or_else(|| panic!("no box char for {:b}", id))
    }
}
