use std::marker::PhantomData;

use crate::salsa::id::AsId;
use crate::salsa::runtime::Runtime;
use crate::salsa::storage::HasJar;

use super::ingredient::Ingredient;
use super::routes::IngredientIndex;

pub trait InternedId: AsId {}
impl<T: AsId> InternedId for T {}

pub trait InternedData: Sized {}
impl<T> InternedData for T {}

#[allow(dead_code)]
pub struct InternedIngredient<Id: InternedId, Data: InternedData> {
    index: IngredientIndex,
    phantom: std::marker::PhantomData<(Id, Data)>,
}

impl<Id, Data> InternedIngredient<Id, Data>
where
    Id: InternedId,
    Data: InternedData,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn intern(&self, runtime: &Runtime, data: Data) -> Id {
        let _ = (runtime, data);
        panic!()
    }

    #[allow(dead_code)]
    pub fn data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        let _ = (runtime, id);
        panic!()
    }
}

impl<Id, Data> Ingredient for InternedIngredient<Id, Data>
where
    Id: InternedId,
    Data: InternedData,
{
    fn maybe_changed_after(
        &self,
        input: super::DatabaseKeyIndex,
        revision: super::Revision,
    ) -> bool {
        todo!()
    }
}
