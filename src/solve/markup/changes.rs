use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map;

use crate::puzzle::{CageId, CellId, Value};
use crate::{HashMap, HashSet};

#[derive(Debug, Default, PartialEq)]
pub(crate) struct PuzzleMarkupChanges {
    pub cells: CellChanges,
    pub cage_solution_removals: HashMap<CageId, Vec<usize>>,
}

impl PuzzleMarkupChanges {
    pub fn clear(&mut self) {
        self.cells.clear();
        self.cage_solution_removals.clear();
    }

    pub fn is_cage_solution_removed(&self, cage_id: CageId, solution_index: usize) -> bool {
        matches!(self.cage_solution_removals.get(&cage_id), Some(values) if values.contains(&solution_index))
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
    DomainRemovals(HashSet<Value>),
    Solution(Value),
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct CellChanges(HashMap<CellId, CellChange>);

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

    pub fn iter(&self) -> <&HashMap<CellId, CellChange> as IntoIterator>::IntoIter {
        self.0.borrow().into_iter()
    }

    pub fn keys(&self) -> hash_map::Keys<'_, CellId, CellChange> {
        self.0.borrow().keys()
    }

    pub fn domain_removals(&self) -> impl Iterator<Item = (CellId, &HashSet<Value>)> {
        self.0.iter().filter_map(|(&id, e)| match e {
            CellChange::DomainRemovals(values) => Some((id, values)),
            _ => None,
        })
    }

    pub fn remove_domain_value(&mut self, id: CellId, value: i32) {
        match self.0.entry(id) {
            hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                // already solved, ignore
                CellChange::Solution(_) => (),
                CellChange::DomainRemovals(values) => {
                    values.insert(value);
                }
            },
            hash_map::Entry::Vacant(entry) => {
                entry.insert(CellChange::DomainRemovals(HashSet::from_iter([value])));
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

impl Borrow<HashMap<CellId, CellChange>> for CellChanges {
    fn borrow(&self) -> &HashMap<CellId, CellChange> {
        &self.0
    }
}

impl BorrowMut<HashMap<CellId, CellChange>> for CellChanges {
    fn borrow_mut(&mut self) -> &mut HashMap<CellId, CellChange> {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a CellChanges {
    type Item = <&'a HashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <&'a HashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut CellChanges {
    type Item = <&'a mut HashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <&'a mut HashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow_mut().into_iter()
    }
}

impl<'a> IntoIterator for CellChanges {
    type Item = <HashMap<CellId, CellChange> as IntoIterator>::Item;
    type IntoIter = <HashMap<CellId, CellChange> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
