use crate::puzzle::{CageId, CellId, Value};
use ahash::AHashMap;
use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct PuzzleMarkupChanges {
    pub cells: CellChanges,
    pub cage_solution_removals: AHashMap<CageId, Vec<usize>>,
}

impl PuzzleMarkupChanges {
    pub fn clear(&mut self) {
        self.cells.clear();
        self.cage_solution_removals.clear();
    }

    pub fn remove_cage_solution(&mut self, cage_id: CageId, solution_index: usize) {
        self.cage_solution_removals
            .entry(cage_id)
            .or_default()
            .push(solution_index);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellChange {
    DomainRemovals(Vec<Value>),
    Solution(Value),
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct CellChanges(AHashMap<CellId, CellChange>);

impl CellChanges {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn get(&self, cell_id: CellId) -> Option<&CellChange> {
        self.0.get(&cell_id)
    }

    pub fn domain_removals(&self) -> impl Iterator<Item = (CellId, &[Value])> {
        self.0.iter().filter_map(|(&id, e)| match e {
            CellChange::DomainRemovals(values) => Some((id, values.as_slice())),
            _ => None,
        })
    }

    pub fn remove_domain_value(&mut self, id: CellId, value: i32) {
        match self.0.entry(id) {
            hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                // already solved, ignore
                CellChange::Solution(_) => (),
                CellChange::DomainRemovals(values) => values.push(value),
            },
            hash_map::Entry::Vacant(entry) => {
                entry.insert(CellChange::DomainRemovals(vec![value]));
            }
        };
    }

    pub fn solutions<'a>(&'a self) -> impl Iterator<Item = (CellId, Value)> + 'a {
        self.0.iter().filter_map(|(&id, e)| match e {
            CellChange::Solution(value) => Some((id, *value)),
            _ => None,
        })
    }

    pub fn solve(&mut self, id: CellId, value: i32) {
        self.0.insert(id, CellChange::Solution(value));
    }
}

impl Borrow<AHashMap<CellId, CellChange>> for CellChanges {
    fn borrow(&self) -> &AHashMap<CellId, CellChange> {
        &self.0
    }
}

impl BorrowMut<AHashMap<CellId, CellChange>> for CellChanges {
    fn borrow_mut(&mut self) -> &mut AHashMap<CellId, CellChange> {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a CellChanges {
    type Item = <&'a AHashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <&'a AHashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut CellChanges {
    type Item = <&'a mut AHashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <&'a mut AHashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow_mut().into_iter()
    }
}

impl<'a> IntoIterator for CellChanges {
    type Item = <AHashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <AHashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
