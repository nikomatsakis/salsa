#[doc(hidden)]
pub mod entity;
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
