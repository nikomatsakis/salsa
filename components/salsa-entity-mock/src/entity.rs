use crate::{
    cycle::CycleRecoveryStrategy,
    ingredient::{Ingredient, MutIngredient},
    interned::{InternedData, InternedId, InternedIngredient},
    key::ActiveDatabaseKeyIndex,
    runtime::Runtime,
    DatabaseKeyIndex, IngredientIndex, Revision,
};

pub trait EntityId: InternedId {}
impl<T: InternedId> EntityId for T {}

pub trait EntityData: InternedData {}
impl<T: InternedData> EntityData for T {}

#[allow(dead_code)]
pub struct EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    interned: InternedIngredient<Id, EntityKey<Data>>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
struct EntityKey<Data> {
    query_key: ActiveDatabaseKeyIndex,
    disambiguator: Disambiguator,
    data: Data,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Disambiguator(pub u32);

impl<Id, Data> EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            interned: InternedIngredient::new(index),
        }
    }

    pub fn new_entity(&self, runtime: &Runtime, data: Data) -> Id {
        let data_hash = crate::hash::hash(&data);
        let (query_key, disambiguator) = runtime.disambiguate_entity(
            self.interned.ingredient_index(),
            self.interned.reset_at(),
            data_hash,
        );
        let entity_key = EntityKey {
            query_key,
            disambiguator,
            data,
        };
        self.interned.intern(runtime, entity_key)
    }

    pub fn entity_data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        &self.interned.data(runtime, id).data
    }

    /// Deletes the given entities. This is used after a query `Q` executes and we can compare
    /// the entities `E_now` that it produced in this revision vs the entities
    /// `E_prev` it produced in the last revision. Any missing entities `E_prev - E_new` can be
    /// deleted.
    ///
    /// # Warning
    ///
    /// Using this method on an entity id that MAY be used in the current revision will lead to
    /// unspecified results (but not UB). See [`InternedIngredient::delete_index`] for more
    /// discussion and important considerations.
    pub(crate) fn delete_entities(&self, _runtime: &Runtime, ids: impl Iterator<Item = Id>) {
        for id in ids {
            self.interned.delete_index(id);
        }
    }
}

impl<DB: ?Sized, Id, Data> Ingredient<DB> for EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    fn maybe_changed_after(&self, db: &DB, input: DatabaseKeyIndex, revision: Revision) -> bool {
        self.interned.maybe_changed_after(db, input, revision)
    }

    fn cycle_recovery_strategy(&self) -> CycleRecoveryStrategy {
        <_ as Ingredient<DB>>::cycle_recovery_strategy(&self.interned)
    }
}

impl<DB: ?Sized, Id, Data> MutIngredient<DB> for EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    fn reset_for_new_revision(&mut self) {
        self.interned.clear_deleted_indices();
    }
}
