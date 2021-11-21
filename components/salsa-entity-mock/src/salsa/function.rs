use std::{hash::Hash, marker::PhantomData};

use super::{AsId, Runtime};

#[allow(dead_code)]
pub struct FunctionIngredients<MP, KM: KeyMap, V: Value> {
    key_map: KM,
    phantom: std::marker::PhantomData<(MP, V)>,
}

pub trait KeyMap: Default {
    type Key: Key;

    fn key_to_index(&self, key: &Self::Key) -> u32;
}

pub struct IdMap<K>(PhantomData<K>);

impl<K> Default for IdMap<K> {
    fn default() -> Self {
        IdMap(PhantomData)
    }
}

impl<K: AsId + Hash + Eq> KeyMap for IdMap<K> {
    type Key = K;

    fn key_to_index(&self, key: &Self::Key) -> u32 {
        key.to_id().as_u32()
    }
}

pub trait Key: Eq + Hash {}
impl<T: Eq + Hash> Key for T {}

pub trait Value: Clone {}
impl<T: Clone> Value for T {}

impl<MP, KM, V> FunctionIngredients<MP, KM, V>
where
    KM: KeyMap,
    V: Value,
{
    pub fn fetch<DB>(
        &self,
        key: KM::Key,
        runtime: &Runtime,
        db: DB,
        compute_value: impl FnOnce(KM::Key, DB) -> V,
    ) -> V {
        let index = self.key_map.key_to_index(&key);
        drop((db, runtime, index, compute_value));
        panic!()
    }
}
