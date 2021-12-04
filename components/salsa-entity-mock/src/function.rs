use crossbeam::atomic::AtomicCell;

use crate::{
    cycle::CycleRecoveryStrategy,
    durability::Durability,
    function::memo::Memo,
    jar::Jar,
    key::{DatabaseKeyIndex, DependencyIndex},
    runtime::local_state::{QueryInputs, QueryRevisions},
    Cycle, DbWithJar, Id, Revision,
};

use super::{ingredient::Ingredient, routes::IngredientIndex, AsId, Runtime};

mod execute;
mod fetch;
mod lru;
mod maybe_changed_after;
mod memo;
mod set;
mod store;
mod sync;

#[allow(dead_code)]
pub struct FunctionIngredient<C: Configuration> {
    index: IngredientIndex,
    memo_map: memo::MemoMap<C::Key, C::Value>,
    sync_map: sync::SyncMap,
    lru: lru::Lru,
}

pub trait Configuration {
    type Jar: Jar;
    type Key: Eq + AsId;
    type Value: Clone + std::fmt::Debug;

    const CYCLE_STRATEGY: CycleRecoveryStrategy;

    const MEMOIZE_VALUE: bool;

    fn should_backdate_value(old_value: &Self::Value, new_value: &Self::Value) -> bool;

    fn execute(db: &DynDb<Self>, key: Self::Key) -> Self::Value;

    fn recover_from_cycle(db: &DynDb<Self>, cycle: &Cycle, key: Self::Key) -> Self::Value;

    fn key_from_id(id: Id) -> Self::Key {
        AsId::from_id(id)
    }
}

pub type DynDb<C> = <<C as Configuration>::Jar as Jar>::DynDb;

impl<C> FunctionIngredient<C>
where
    C: Configuration,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            index,
            memo_map: memo::MemoMap::default(),
            lru: Default::default(),
            sync_map: Default::default(),
        }
    }

    fn database_key_index(&self, k: C::Key) -> DatabaseKeyIndex {
        DatabaseKeyIndex {
            ingredient_index: self.index,
            key_index: k.as_id(),
        }
    }
}

impl<DB, C> Ingredient<DB> for FunctionIngredient<C>
where
    DB: ?Sized + DbWithJar<C::Jar>,
    C: Configuration,
{
    fn maybe_changed_after(&self, db: &DB, input: DependencyIndex, revision: Revision) -> bool {
        let key = C::key_from_id(input.key_index.unwrap());
        let db = db.as_jar_db();
        self.maybe_changed_after(db, key, revision)
    }

    fn cycle_recovery_strategy(&self) -> CycleRecoveryStrategy {
        C::CYCLE_STRATEGY
    }
}
