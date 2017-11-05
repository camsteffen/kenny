use std::ops::Deref;
use range_domain::RangeDomain;
use std::ops::{Index, IndexMut};
use square::vector::VectorId;

pub struct VectorValueDomainSet {
    size: usize,
    set: Vec<Vec<Option<RangeDomain>>>,
}

impl VectorValueDomainSet {
    pub fn new(size: usize) -> Self {
        Self {
            size: size,
            set: vec![vec![Some(RangeDomain::with_all(size)); size]; 2 * size],
        }
    }

    pub fn remove_vector_value(&mut self, vector_id: VectorId, value: i32) {
        self[vector_id][value as usize - 1] = None;
    }
}

impl Deref for VectorValueDomainSet {
    type Target = Vec<Vec<Option<RangeDomain>>>;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl Index<VectorId> for VectorValueDomainSet {
    type Output = Vec<Option<RangeDomain>>;

    fn index<'a>(&'a self, vector_id: VectorId) -> &'a Self::Output {
        let index = vector_id.as_number(self.size);
        &self.set[index]
    }
}

impl IndexMut<VectorId> for VectorValueDomainSet {
    fn index_mut<'a>(&'a mut self, vector_id: VectorId) -> &'a mut Self::Output {
        let index = vector_id.as_number(self.size);
        &mut self.set[index]
    }
}