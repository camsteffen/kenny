use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use puzzle::solve::StateWriter;
use puzzle::solve::markup::PuzzleMarkupChanges;
use super::CageSolutionsConstraint;
use super::CageVectorValueConstraint;
use super::Constraint;
use super::VectorSolvedCellConstraint;
use super::VectorSubdomainConstraint;
use super::VectorValueDomainConstraint;
use super::ConstraintSet;

pub fn constraint_propogation(puzzle: &Puzzle, markup: &mut PuzzleMarkup) {
    let mut state_writer = StateWriter::new();
    state_writer.write(puzzle, markup);
    markup.init_cage_solutions(puzzle);

    let mut constraints = ConstraintSet::new(puzzle);
    let mut changes = PuzzleMarkupChanges::new();

    let mut loop_count = 0;
    loop {
        let has_changes = (0..ConstraintSet::len()).any(|i|
            constraints.select_map(i, |c| c.enforce_partial(puzzle, markup, &mut changes)));
        if !has_changes { break }
        markup.sync_changes(&mut changes);
        constraints.for_each(|c| c.notify_changes(&changes));
        changes.clear();
        state_writer.write(puzzle, markup);
        loop_count += 1;
    }
}

pub fn constraint_propogation_old(puzzle: &Puzzle, markup: &mut PuzzleMarkup) {
    let mut state_writer = StateWriter::new();
    state_writer.write(puzzle, markup);
    markup.init_cage_solutions(puzzle);

    let mut constraints = default_constraint_set(puzzle);
    let mut changes = PuzzleMarkupChanges::new();

    let mut loop_count = 0;
    loop {
        for constraint in &mut constraints {
            if constraint.enforce_partial(puzzle, markup, &mut changes) {
                break
            }
        }
        if changes.is_empty() { break }
        markup.sync_changes(&mut changes);
        for constraint in &mut constraints {
            constraint.notify_changes(&changes);
        }
        changes.clear();
        state_writer.write(puzzle, markup);
        loop_count += 1;
    }
}

fn default_constraint_set(puzzle: &Puzzle) -> Vec<Box<Constraint>> {
    vec![
        Box::new(VectorSolvedCellConstraint::new()),
        Box::new(CageSolutionsConstraint::new(puzzle)),
        Box::new(CageVectorValueConstraint::new(puzzle)),
        Box::new(VectorSubdomainConstraint::new(puzzle.width)),
        Box::new(VectorValueDomainConstraint::new(puzzle.width)),
    ]
}
