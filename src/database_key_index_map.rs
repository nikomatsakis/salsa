use crate::plumbing::HasQueryGroup;

use crate::{Database, DatabaseKeyIndex, Query};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

pub(crate) struct DatabaseKeyIndexMap<DB, Q>
where
    Q: Query<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
{
    map: RwLock<FxHashMap<Q::Key, DatabaseKeyIndex>>,
}

impl<DB, Q> Default for DatabaseKeyIndexMap<DB, Q>
where
    Q: Query<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<DB, Q> DatabaseKeyIndexMap<DB, Q>
where
    Q: Query<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
{
    pub(crate) fn insert(&self, db: &DB, key: &Q::Key) -> DatabaseKeyIndex {
        if let Some(index) = self.map.read().get(key) {
            return *index;
        }

        *self.map.write().entry(key.clone()).or_insert_with(|| {
            let group_key = Q::group_key(key.clone());
            DB::create_database_key_index(db, group_key)
        })
    }
}
