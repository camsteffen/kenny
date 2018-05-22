use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use puzzle::solve::StepWriter;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::solve::constraint::Constraint;
use puzzle::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use puzzle::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use puzzle::solve::constraint::cage_solutions::CageSolutionsConstraint;
use puzzle::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use puzzle::solve::constraint::vector_subdomain::VectorSubdomainConstraint;
use puzzle::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use puzzle::solve::constraint::cage_solution_vector_domain::CageSolutionVectorDomainConstraint;

pub fn constraint_propagation(puzzle: &Puzzle, markup: &mut PuzzleMarkup,
                              changes: &mut PuzzleMarkupChanges, mut step_writer: Option<&mut StepWriter>) {
    markup.init_cage_solutions(puzzle);

    let mut constraints = constraint_set(puzzle);

    let mut loop_count = 0;
    loop {
        for c in &mut constraints {
            c.notify_changes(puzzle, changes);
        }
        changes.clear();
        let has_changes = constraints.iter_mut().any(|c|
            c.enforce_partial(puzzle, markup, changes));
        if !has_changes { break }
        markup.sync_changes(changes);
        if let Some(step_writer) = &mut step_writer {
            let changed_cells = changes.cell_domain_value_removals.keys().cloned().collect::<Vec<_>>();
            step_writer.write_next(puzzle, markup, &changed_cells);
        }
        loop_count += 1;
        if markup.is_solved() { break }
    }
    debug!("constraint propagation finished after {} iterations", loop_count);
}

fn constraint_set(puzzle: &Puzzle) -> Vec<Box<Constraint>> {
    vec![
        Box::new(VectorSolvedCellConstraint::new()),
        Box::new(VectorValueDomainConstraint::new(puzzle.width)),
        Box::new(CageSolutionsConstraint::new(puzzle)),
        Box::new(CageVectorValueConstraint::new(puzzle)),
        Box::new(VectorSubdomainConstraint::new()),
        Box::new(VectorValueCageConstraint::new(puzzle)),
        Box::new(CageSolutionVectorDomainConstraint::new(puzzle)),
    ]
}
