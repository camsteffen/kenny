pub mod range_set;
pub mod square;

pub use self::range_set::RangeSet;
pub use self::square::Square;

use linked_hash_set::LinkedHashSet;

pub type LinkedAHashSet<T> = LinkedHashSet<T, ahash::RandomState>;
