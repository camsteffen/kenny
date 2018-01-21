use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use puzzle::solve::StateWriter;
use puzzle::solve::markup::PuzzleMarkupChanges;
use super::ConstraintSet;
use std::path::Path;

pub fn constraint_propogation(puzzle: &Puzzle, markup: &mut PuzzleMarkup,
                              changes: &mut PuzzleMarkupChanges, step_images_path: Option<&Path>) {
    let mut state_writer = step_images_path.map(|path| {
        let mut state_writer = StateWriter::new(path);
        state_writer.write_next(puzzle, markup, &[]);
        state_writer
    });
    markup.init_cage_solutions(puzzle);

    let mut constraints = ConstraintSet::new(puzzle);

    let mut loop_count = 0;
    loop {
        constraints.for_each(|c| c.notify_changes(puzzle, &changes));
        changes.clear();
        let has_changes = (0..ConstraintSet::len()).any(|i|
            constraints.select_map(i, |c| c.enforce_partial(puzzle, markup, changes)));
        if !has_changes { break }
        markup.sync_changes(changes);
        if let Some(state_writer) = state_writer.as_mut() {
            let changed_cells = changes.cell_domain_value_removals.keys().cloned().collect::<Vec<_>>();
            state_writer.write_next(puzzle, markup, &changed_cells);
        }
        loop_count += 1;
        if markup.is_solved() { break }
    }
}
