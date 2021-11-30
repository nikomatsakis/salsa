use std::{hash::Hash, marker::PhantomData};

use super::{ingredient::Ingredient, routes::IngredientIndex, AsId, Runtime};

mod memo;

#[allow(dead_code)]
pub struct FunctionIngredient<K: Key, V: Value> {
    index: IngredientIndex,
    memo_map: memo::MemoMap<K, V>,
}

pub trait Key: Eq + AsId {}
impl<T: AsId> Key for T {}

pub trait Value: Clone {}
impl<T: Clone> Value for T {}

impl<K, V> FunctionIngredient<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new(index: IngredientIndex) -> Self {
        Self {
            index,
            memo_map: memo::MemoMap::default(),
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

    pub fn store<DB>(&mut self, key: K, runtime: &mut Runtime, db: DB, value: V) {
        todo!()
    }
}

impl<K, V> Ingredient for FunctionIngredient<K, V>
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
