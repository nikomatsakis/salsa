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

impl DatabaseKeyIndex {
    /// Create a database-key-index for an interning or entity table.
    /// The `key_index` here is always zero, which deliberately corresponds to
    /// no particular id or entry. This is because the data in such tables
    /// remains valid until the table as a whole is reset. Using a single id avoids
    /// creating tons of dependencies in the dependency listings.
    pub(super) fn for_table(ingredient_index: IngredientIndex) -> Self {
        Self {
            ingredient_index,
            key_index: 0,
        }
    }
}
