#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn bench_puzzles(b: &mut Bencher) {
    b.iter(|| puzzle_tests::test_all_puzzles());
}

// test bench_puzzles ... bench: 566,020,715 ns/iter (+/- 110,910,756)
