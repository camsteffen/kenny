use std::path::{Path, PathBuf};

pub fn test_all_puzzles() {
    test_puzzle_dir(project_path("res/test/puzzles/require-search"), true);
    test_puzzle_dir(project_path("res/test/puzzles/no-require-search"), false);
}

fn test_puzzle_dir(path: impl AsRef<Path>, require_search: bool) {
    let mut files: Vec<_> = fs::read_dir(path).unwrap().map(|f| f.unwrap()).collect();
    files.sort_unstable_by_key(|f| f.path());
    for file in files {
        println!("Solving {}", file.path().display());
        let puzzle = Puzzle::from_file(&file.path()).unwrap();
        let solve_result = PuzzleSolver::new(&puzzle).solve().unwrap();
        let data = solve_result.solved().unwrap();
        assert_eq!(
            data.used_search,
            require_search,
            "{}",
            file.path().display()
        );
    }
}

fn project_path(path: impl AsRef<Path>) -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push(path);
    root
}
