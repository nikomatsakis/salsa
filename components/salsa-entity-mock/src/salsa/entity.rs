use crate::salsa::runtime::Runtime;

use super::{
    ingredient::Ingredient,
    interned::{InternedData, InternedId, InternedIngredient},
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
    query_key: DatabaseKeyIndex,
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

    #[allow(dead_code)]
    pub fn new_entity(&self, runtime: &Runtime, data: Data) -> Id {
        let data_hash = crate::salsa::hash::hash(&data);
        let (query_key, disambiguator) =
            runtime.disambiguate_entity(self.interned.ingredient_index(), data_hash);
        let entity_key = EntityKey {
            query_key,
            disambiguator,
            data,
        };
        self.interned.intern(runtime, entity_key)
    }

    #[allow(dead_code)]
    pub fn entity_data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        &self.interned.data(runtime, id).data
    }
}

impl<Id, Data> Ingredient for EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    fn maybe_changed_after(&self, input: DatabaseKeyIndex, revision: Revision) -> bool {
        self.interned.maybe_changed_after(input, revision)
    }
}
