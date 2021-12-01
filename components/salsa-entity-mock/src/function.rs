use std::{hash::Hash, marker::PhantomData};

use crossbeam::atomic::AtomicCell;

use crate::{
    durability::Durability,
    function::memo::Memo,
    runtime::local_state::{QueryInputs, QueryRevisions},
};

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

    pub fn store(&mut self, key: K, runtime: &mut Runtime, value: V, durability: Durability) {
        let revision = runtime.current_revision();
        let memo = Memo {
            value: Some(value),
            verified_at: AtomicCell::new(revision),
            revisions: QueryRevisions {
                changed_at: revision,
                durability,
                inputs: QueryInputs::Tracked {
                    inputs: runtime.empty_dependencies(),
                },
            },
        };

        if let Some(old_value) = self.memo_map.insert(key, memo) {
            let durability = old_value.load().revisions.durability;
            runtime.report_tracked_write(durability);
        }
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
