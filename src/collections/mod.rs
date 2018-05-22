pub mod range_set;
pub mod square;

pub use self::range_set::RangeSet;
pub use self::square::Square;

use fnv::FnvBuildHasher;
use linked_hash_set::LinkedHashSet;

pub type FnvLinkedHashSet<T> = LinkedHashSet<T, FnvBuildHasher>;
