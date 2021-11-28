pub mod database;
#[doc(hidden)]
pub mod entity;
pub mod function;
pub mod hash;
pub mod id;
pub mod ingredient;
pub mod interned;
pub mod jar;
pub mod plumbing;
pub mod prelude;
pub mod revision;
pub mod routes;
pub mod runtime;
pub mod storage;

pub use self::entity::EntityData;
pub use self::entity::EntityId;
pub use self::id::AsId;
pub use self::id::Id;
pub use self::revision::Revision;
pub use self::routes::IngredientIndex;
pub use self::runtime::Runtime; // FIXME
pub use self::storage::HasJar;
pub use self::storage::Storage;

pub trait Database {}

pub trait ParallelDatabase: Database {}

/// An integer that uniquely identifies a particular query instance within the
/// database. Used to track dependencies between queries. Fully ordered and
/// equatable but those orderings are arbitrary, and meant to be used only for
/// inserting into maps and the like.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DatabaseKeyIndex {
    ingredient_index: IngredientIndex,
    key_index: u32,
}
