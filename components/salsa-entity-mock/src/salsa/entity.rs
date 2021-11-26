use std::marker::PhantomData;

use crate::salsa::id::AsId;
use crate::salsa::runtime::Runtime;

use super::{ingredient::Ingredient, DatabaseKeyIndex, IngredientIndex, Revision};

pub trait EntityId: AsId {}
impl<T: AsId> EntityId for T {}

pub trait EntityData: Sized {}
impl<T> EntityData for T {}

#[allow(dead_code)]
pub struct EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    index: IngredientIndex,
    phantom: std::marker::PhantomData<(Id, Data)>,
}

impl<Id, Data> EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn new_entity(&self, runtime: &Runtime, data: Data) -> Id {
        let _ = (runtime, data);
        todo!()
    }

    #[allow(dead_code)]
    pub fn entity_data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        let _ = (runtime, id);
        todo!()
    }
}

impl<Id, Data> Ingredient for EntityIngredient<Id, Data>
where
    Id: EntityId,
    Data: EntityData,
{
    fn maybe_changed_after(&self, key: DatabaseKeyIndex, revision: Revision) -> bool {
        todo!()
    }
}
