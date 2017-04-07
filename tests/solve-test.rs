use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use kenny::puzzle::Puzzle;
use kenny::solve::PuzzleSolver;

#[test]
fn test_puzzles() -> Result<()> {
    test_puzzle_dir(project_path("res/test/puzzles/require-search"), true)?;
    test_puzzle_dir(project_path("res/test/puzzles/no-require-search"), false)?;
    Ok(())
}

fn test_puzzle_dir(path: impl AsRef<Path>, require_search: bool) -> Result<()> {
    let mut files: Vec<_> = fs::read_dir(path).unwrap().map(|f| f.unwrap()).collect();
    files.sort_unstable_by_key(|f| f.path());
    for file in files {
        println!("Solving {}", file.path().display());
        let puzzle = Puzzle::from_file(&file.path()).unwrap();
        let solve_result = PuzzleSolver::new(&puzzle).solve()?;
        assert!(
            solve_result.is_solved(),
            "Could not solve {}",
            file.path().display()
        );
        let data = solve_result.solved().unwrap();
        assert_eq!(
            data.used_search,
            require_search,
            "{}",
            file.path().display()
        );
    }
    Ok(())
}

fn project_path(path: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
