use std::path::{Path, PathBuf};

use clap::{Arg, ArgGroup, ArgMatches};
use failure::{err_msg, Fallible};

use crate::{DEFAULT_PATH, DEFAULT_PUZZLE_WIDTH};

#[derive(Clone)]
pub struct Options {
    pub image_width: Option<u32>,
    output_path: Option<PathBuf>,
    source: Source,
    solve: Option<Solve>,
    save_image: bool,
}

impl Options {
    pub fn from_args() -> Fallible<Self> {
        Self::from_arg_matches(&clap_app().get_matches())
    }

    fn from_arg_matches(matches: &ArgMatches) -> Fallible<Self> {
        let mut options = Self {
            image_width: matches.value_of("image_width")
                .map(|s| s.parse().expect("invalid image width")),
            output_path: None,
            source: match matches.value_of("input") {
                Some(path) => Source::File(path.to_owned()),
                None => {
                    let exclude_solvable = matches.is_present("exclude-unsolvable");
                    let include_unsolvable = matches.is_present("include-unsolvable");
                    if exclude_solvable && !include_unsolvable {
                        return Err(err_msg("All puzzles are excluded"));
                    }
                    Source::Generate(Generate {
                        count: matches.value_of("count")
                            .map(|s| s.parse::<u32>().expect("invalid count"))
                            .unwrap_or(1),
                        width: matches.value_of("width")
                            .map(|s| s.parse::<usize>().expect("invalid width"))
                            .unwrap_or(DEFAULT_PUZZLE_WIDTH),
                        save_puzzle: matches.is_present("save_puzzle"),
                        exclude_solvable,
                        include_unsolvable,
                    })
                }
            },
            solve: if matches.is_present("solve") {
                Some(Solve {
                    save_image: matches.is_present("save_solved_image"),
                    save_step_images: matches.is_present("save_step_images"),
                })
            } else { None },
            save_image: matches.is_present("save_image"),
        };
        if options.save_any() {
            options.output_path = Some(matches.value_of("output_path").unwrap().into())
        } else if matches.occurrences_of("output_path") != 0 {
            return Err(err_msg("output path specified but nothing to save"));
        }
        Ok(options)
    }


    /// returns true if any files are to be saved
    pub fn save_any(&self) -> bool {
        if self.save_image {
            return true;
        }
        if let Source::Generate(ref gc) = &self.source {
            if gc.save_puzzle {
                return true;
            }
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
}

#[derive(Clone)]
pub enum Source {
    File(String),
    Generate(Generate),
}

impl Source {
    /*
    pub fn file(&self) -> Option<&str> {
        match self {
            Source::File(f) => Some(f),
            _ => None,
        }
    }
    */

    pub fn generate(&self) -> Option<&Generate> {
        match self {
            Source::Generate(g) => Some(g),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Generate {
    pub count: u32,
    pub width: usize,
    pub save_puzzle: bool,
    pub exclude_solvable: bool,
    pub include_unsolvable: bool,
}

#[derive(Clone)]
pub struct Solve {
    pub save_image: bool,
    pub save_step_images: bool,
}


fn clap_app<'a>() -> clap::App<'a, 'a> {
    clap::App::new("CamCam")
        .author("Cameron Steffen <cam.steffen94@gmail.com>")
        .about("Solve KenKen Puzzles")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .group(ArgGroup::with_name("source")
            .args(&["generate", "input"])
            .required(true))
        .arg(Arg::with_name("generate")
            .short("g")
            .long("generate")
            .help("generate KenKen puzzle(s)"))
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .takes_value(true)
            .value_name("PATH")
            .help("read a KenKen puzzle from a file"))
        .arg(Arg::with_name("solve")
            .short("s")
            .long("solve")
            .help("solve KenKen puzzle(s)"))
        .arg(Arg::with_name("width")
            .short("w")
            .long("width")
            .takes_value(true)
            .value_name("WIDTH")
            .requires("generate")
            .help("set the width and height of the generated puzzle"))
        .arg(Arg::with_name("output")
            .long("output-path")
            .help("directory to save files")
            .default_value(DEFAULT_PATH))
        .arg(Arg::with_name("count")
            .short("c")
            .long("count")
            .requires("generate")
            .takes_value(true)
            .help("the number of puzzles to generate (and solve)"))
        .arg(Arg::with_name("exclude_solvable")
            .long("exclude-solvable")
            .requires("generate")
            .help("exclude solvable generated puzzles"))
        .arg(Arg::with_name("include_unsolvable")
            .long("include-unsolvable")
            .requires("generate")
            .help("include unsolvable generated puzzles"))
        .arg(Arg::with_name("save_puzzle")
            .long("save-puzzle")
            .requires("generate")
            .help("save the puzzle to a file"))
        .arg(Arg::with_name("save_image")
            .long("save-image")
            .help("save an image of the puzzle(s)"))
        .arg(Arg::with_name("save_solved_image")
            .long("save-solved-image")
            .requires("solve")
            .help("save an image of the solved (or partially solved) puzzle(s)"))
        .arg(Arg::with_name("save_step_images")
            .long("save-step-images")
            .help("save an image of the puzzle at each step of the solving process"))
        .arg(Arg::with_name("image_width")
            .long("image-width")
            .takes_value(true)
            .value_name("PIXELS")
            .help("sets the approx. width of saved images in pixels"))
}
