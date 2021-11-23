use std::{hash::Hash, marker::PhantomData};

use super::{AsId, Runtime};

#[allow(dead_code)]
pub struct FunctionIngredients<MP, K: Key, V: Value> {
    phantom: std::marker::PhantomData<(MP, K, V)>,
}

pub trait Key: Eq + Hash + AsId {}
impl<T: Eq + Hash + AsId> Key for T {}

pub trait Value: Clone {}
impl<T: Clone> Value for T {}

impl<MP, K, V> FunctionIngredients<MP, K, V>
where
    K: Key,
    V: Value,
{
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
