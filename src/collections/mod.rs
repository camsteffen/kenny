pub mod square;

pub(crate) mod iterator_ext;
pub(crate) mod range_set;
pub(crate) mod vec_ext;

use linked_hash_set::LinkedHashSet;

pub type LinkedAHashSet<T> = LinkedHashSet<T, ahash::RandomState>;
