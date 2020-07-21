#![cfg(feature = "test")]
#![feature(test)]

extern crate test;

use camcam::puzzle::solve::PuzzleSolver;
use camcam::puzzle::Puzzle;
use std::fs;
use std::path::{Path, PathBuf};
use test::Bencher;

#[test]
pub fn test_all_puzzles() {
    puzzle_tests::test_all_puzzles();
}
