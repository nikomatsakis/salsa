#[doc(hidden)]
pub mod entity;
pub mod function;
mod id;
mod interned;
pub mod plumbing;
pub mod prelude;
mod runtime;
mod storage;

pub use self::entity::EntityData;
pub use self::entity::EntityId;
pub use self::id::AsId;
pub use self::id::Id;
pub use self::plumbing::HasJar;
pub use self::runtime::Runtime; // FIXME

pub trait Database {}

pub trait ParallelDatabase: Database {}

/// An integer that uniquely identifies a particular query instance within the
/// database. Used to track dependencies between queries. Fully ordered and
/// equatable but those orderings are arbitrary, and meant to be used only for
/// inserting into maps and the like.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DatabaseKeyIndex {
    group_index: u16,
    query_index: u16,
    key_index: u32,
}
