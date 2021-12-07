use crate::{
    cycle::CycleRecoveryStrategy,
    jar::Jar,
    key::{DatabaseKeyIndex, DependencyIndex},
    Cycle, DbWithJar, Id, Revision,
};

use super::{ingredient::Ingredient, routes::IngredientIndex, AsId};

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
    type Jar: for<'db> Jar<'db>;
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

/// True if `old_value == new_value`. Invoked by the generated
/// code for `should_backdate_value` so as to give a better
/// error message.
pub fn should_backdate_value<V: Eq>(old_value: &V, new_value: &V) -> bool {
    old_value == new_value
}

pub type DynDb<'bound, C> = <<C as Configuration>::Jar as Jar<'bound>>::DynDb;

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

    pub fn set_capacity(&self, capacity: usize) {
        self.lru.set_capacity(capacity);
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
