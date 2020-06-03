pub mod iterator_ext;
pub mod range_set;
pub mod square;
pub mod vec_ext;

use linked_hash_set::LinkedHashSet;

pub type LinkedAHashSet<T> = LinkedHashSet<T, ahash::RandomState>;
