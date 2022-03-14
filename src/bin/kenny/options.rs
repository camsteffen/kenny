use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use kenny::collections::square::SquareValue;

const DEFAULT_PUZZLE_WIDTH: SquareValue = 4;
const DEFAULT_PATH: &str = "output";

#[derive(Clone)]
pub(crate) struct Options {
    output_path: Option<PathBuf>,
    source: Source,
    solve: Option<Solve>,
    save_image: bool,
    save_puzzle: bool,
}

impl Options {
    pub fn from_args() -> Result<Self> {
        Self::from_arg_matches(&clap_app().get_matches())
    }

    fn from_arg_matches(matches: &ArgMatches<'_>) -> Result<Self> {
        let save_all = matches.is_present("save_all");
        let mut options = Self {
            output_path: None,
            source: if let Some(path) = matches.value_of("input") {
                Source::File(path.into())
            } else {
                let (include_solvable, include_unsolvable) =
                    if matches.is_present("allow_unsolvable") {
                        (true, true)
                    } else if matches.is_present("unsolvable_only") {
                        (false, true)
                    } else {
                        (true, false)
                    };
                Source::Generate(Generate {
                    count: matches
                        .value_of("count")
                        .map_or(1, |s| s.parse::<u32>().expect("invalid count")),
                    width: matches.value_of("width").map_or(DEFAULT_PUZZLE_WIDTH, |s| {
                        s.parse::<SquareValue>().expect("invalid width")
                    }),
                    include_solvable,
                    include_unsolvable,
                    require_search: matches.is_present("require_search"),
                    no_require_search: matches.is_present("no_require_search"),
                })
            },
            solve: if matches.is_present("solve") {
                Some(Solve {
                    save_image: matches.is_present("save_solved_image") || save_all,
                    save_step_images: matches.is_present("save_step_images") || save_all,
                })
            } else {
                None
            },
            save_image: matches.is_present("save_image") || save_all,
            save_puzzle: matches.is_present("save_puzzle") || save_all,
        };
        if options.save_any() {
            options.output_path = Some(matches.value_of("output_path").unwrap().into())
        } else if matches.occurrences_of("output_path") != 0 {
            anyhow!("output path specified but nothing to save");
        }
        Ok(options)
    }

    /// returns true if any files are to be saved
    pub fn save_any(&self) -> bool {
        if self.save_image || self.save_puzzle {
            return true;
        }
        if let Some(ref sc) = &self.solve {
            if sc.save_image || sc.save_step_images {
                return true;
            }
        }
        false
    }

    pub fn output_path(&self) -> Option<&Path> {
        self.output_path.as_deref()
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn solve(&self) -> Option<&Solve> {
        self.solve.as_ref()
    }

    pub fn save_image(&self) -> bool {
        self.save_image
    }

    pub fn save_puzzle(&self) -> bool {
        self.save_puzzle
    }
}

#[derive(Clone)]
pub(crate) enum Source {
    File(PathBuf),
    Generate(Generate),
}

impl Source {
    pub fn file(&self) -> Option<&Path> {
        match self {
            Source::File(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Generate {
    pub count: u32,
    pub width: SquareValue,
    pub include_solvable: bool,
    pub include_unsolvable: bool,
    pub require_search: bool,
    pub no_require_search: bool,
}

#[derive(Clone)]
pub(crate) struct Solve {
    pub save_image: bool,
    pub save_step_images: bool,
}

fn clap_app() -> clap::App<'static, 'static> {
    use clap::{App, AppSettings, Arg, ArgGroup};

    App::new("Kenny")
        .author("Cameron Steffen <cam.steffen94@gmail.com>")
        .help_message("Solve KenKen Puzzles")
        .setting(AppSettings::ArgRequiredElseHelp)
        // can use in clap 3.0 when released
        // .replace("--save-all", &["--save-puzzle", "--save-image", "--save-solved-image", "--save-step-images"])
        .group(
            ArgGroup::with_name("source")
                .args(&["generate", "input"])
                .required(true),
        )
        .arg(
            Arg::with_name("generate")
                .short("g")
                .long("generate")
                .help("generate KenKen puzzle(s)")
                .display_order(1),
        )
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .takes_value(true)
                .value_name("PATH")
                .help("read a KenKen puzzle from a file")
                .display_order(1),
        )
        .arg(
            Arg::with_name("solve")
                .short("s")
                .long("solve")
                .help("solve KenKen puzzle(s)"),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .takes_value(true)
                .value_name("WIDTH")
                .requires("generate")
                .help("set the width and height of the generated puzzle"),
        )
        .arg(
            Arg::with_name("output_path")
                .long("output-path")
                .short("o")
                .help("directory to save files")
                .default_value(DEFAULT_PATH),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .requires("generate")
                .takes_value(true)
                .help("the number of puzzles to generate (and solve)"),
        )
        .arg(
            Arg::with_name("require_search")
                .long("require-search")
                .requires("generate")
                .help("only include puzzles that require backtracking search to solve"),
        )
        .arg(
            Arg::with_name("no_require_search")
                .long("no-require-search")
                .requires("generate")
                .conflicts_with("require_search")
                .help("only include puzzles that do not require backtracking search to solve"),
        )
        .arg(
            Arg::with_name("allow_unsolvable")
                .long("allow-unsolvable")
                .requires("generate")
                .help("include unsolvable generated puzzles"),
        )
        .arg(
            Arg::with_name("unsolvable_only")
                .long("unsolvable-only")
                .requires("generate")
                .conflicts_with("allow_unsolvable")
                .help("exclude solvable generated puzzles"),
        )
        .arg(
            Arg::with_name("save_all")
                .long("save-all")
                .help("save all optional files"),
        )
        .arg(
            Arg::with_name("save_puzzle")
                .long("save-puzzle")
                .help("save the puzzle to a file"),
        )
        .arg(
            Arg::with_name("save_image")
                .long("save-image")
                .help("save an image of the puzzle(s)"),
        )
        .arg(
            Arg::with_name("save_solved_image")
                .long("save-solved-image")
                .requires("solve")
                .help("save an image of the solved (or partially solved) puzzle(s)"),
        )
        .arg(
            Arg::with_name("save_step_images")
                .long("save-step-images")
                .help("save an image of the puzzle at each step of the solving process"),
        )
}
