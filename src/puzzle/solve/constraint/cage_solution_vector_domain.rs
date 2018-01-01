use super::Constraint;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use puzzle::solve::PuzzleMarkupChanges;

struct CageSolutionVectorDomainConstraint {
}

impl CageSolutionVectorDomainConstraint {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Constraint for CageSolutionVectorDomainConstraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        false
    }

    fn notify_changes(&mut self, changes: &PuzzleMarkupChanges) {

    }
}
