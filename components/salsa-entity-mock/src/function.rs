use std::{hash::Hash, marker::PhantomData};

use super::{ingredient::Ingredient, routes::IngredientIndex, AsId, Runtime};

#[allow(dead_code)]
pub struct FunctionIngredient<MP, K: Key, V: Value> {
    index: IngredientIndex,
    phantom: std::marker::PhantomData<(MP, K, V)>,
}

pub trait Key: Eq + Hash + AsId {}
impl<T: Eq + Hash + AsId> Key for T {}

pub trait Value: Clone {}
impl<T: Clone> Value for T {}

impl<MP, K, V> FunctionIngredient<MP, K, V>
where
    K: Key,
    V: Value,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    pub fn fetch<DB>(
        &self,
        key: K,
        runtime: &Runtime,
        db: DB,
        compute_value: impl FnOnce(K, DB) -> V,
    ) -> V {
        let index = key.as_id().as_u32();
        drop((db, runtime, index, compute_value));
        panic!()
    }
}

impl<MP, K, V> Ingredient for FunctionIngredient<MP, K, V>
where
    K: Key,
    V: Value,
{
    fn maybe_changed_after(
        &self,
        _input: super::DatabaseKeyIndex,
        _revision: super::Revision,
    ) -> bool {
        todo!()
    }
}
