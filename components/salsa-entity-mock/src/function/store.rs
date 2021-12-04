use crossbeam::atomic::AtomicCell;

use crate::{
    durability::Durability,
    entity::EntityInDb,
    key::DependencyIndex,
    runtime::local_state::{QueryInputs, QueryRevisions},
    Database, DatabaseKeyIndex, Runtime,
};

use super::{memo::Memo, Configuration, DynDb, FunctionIngredient};

impl<C> FunctionIngredient<C>
where
    C: Configuration,
{
    pub fn store(
        &mut self,
        key: C::Key,
        runtime: &mut Runtime,
        value: C::Value,
        durability: Durability,
    ) {
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
