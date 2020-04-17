use DEFAULT_PUZZLE_WIDTH;

pub struct Options {
    pub image_width: Option<u32>,
    pub output_path: Option<String>,
    pub source: Source,
    pub solve: Option<Solve>,
    pub save_image: bool,
}

impl Options {
    pub fn from_args() -> Self {
        Self::from_arg_matches(&clap_app().get_matches())
    }

    fn from_arg_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            image_width: matches.value_of("image_width")
                .map(|s| s.parse().expect("invalid image width")),
            output_path: matches.value_of("output_path").map(|s| s.to_owned()),
            source: match matches.value_of("input") {
                Some(path) => Source::File(path.to_owned()),
                None => Source::Generate(Generate {
                    count: matches.value_of("count")
                        .map(|s| s.parse::<u32>().expect("invalid count"))
                        .unwrap_or(1),
                    width: matches.value_of("width")
                        .map(|s| s.parse::<u32>().expect("invalid width"))
                        .unwrap_or(DEFAULT_PUZZLE_WIDTH),
                    save_puzzle: matches.is_present("save_puzzle"),
                    unsolvable: matches.is_present("unsolvable"),
                }),
            },
            solve: if matches.is_present("solve") {
                Some(Solve {
                    save_image: matches.is_present("save_solved_image"),
                    save_step_images: matches.is_present("save_step_images"),
                })
            } else { None },
            save_image: matches.is_present("save_image"),
        }
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
}

pub enum Source {
    File(String),
    Generate(Generate),
}

pub struct Generate {
    pub count: u32,
    pub width: u32,
    pub save_puzzle: bool,
    pub unsolvable: bool,
}

pub struct Solve {
    pub save_image: bool,
    pub save_step_images: bool,
}


fn clap_app<'a>() -> clap::App<'a, 'a> {
    clap::App::new("CamCam")
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
}
