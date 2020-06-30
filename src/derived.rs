use crate::debug::TableEntry;
use crate::durability::Durability;
use crate::lru::Lru;
use crate::plumbing::DerivedQueryStorageOps;
use crate::plumbing::HasQueryGroup;
use crate::plumbing::LruQueryStorageOps;
use crate::plumbing::QueryFunction;
use crate::plumbing::QueryStorageMassOps;
use crate::plumbing::QueryStorageOps;
use crate::runtime::StampedValue;
use crate::{
    database_key_index_map::DatabaseKeyIndexMap, CycleError, Database, DatabaseKeyIndex,
    SweepStrategy,
};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;
use std::sync::Arc;

mod slot;
use slot::Slot;

/// Memoized queries store the result plus a list of the other queries
/// that they invoked. This means we can avoid recomputing them when
/// none of those inputs have changed.
pub type MemoizedStorage<DB, Q> = DerivedStorage<DB, Q, AlwaysMemoizeValue>;

/// "Dependency" queries just track their dependencies and not the
/// actual value (which they produce on demand). This lessens the
/// storage requirements.
pub type DependencyStorage<DB, Q> = DerivedStorage<DB, Q, NeverMemoizeValue>;

/// Handles storage where the value is 'derived' by executing a
/// function (in contrast to "inputs").
pub struct DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    lru_list: Lru<DatabaseKeyIndex>,
    database_key_indices: DatabaseKeyIndexMap<DB, Q>,
    slot_map: RwLock<FxHashMap<DatabaseKeyIndex, Arc<Slot<DB, Q, MP>>>>,
    policy: PhantomData<MP>,
}

impl<DB, Q, MP> std::panic::RefUnwindSafe for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
    Q::Key: std::panic::RefUnwindSafe,
    Q::Value: std::panic::RefUnwindSafe,
{
}

pub trait MemoizationPolicy<DB, Q>: Send + Sync + 'static
where
    Q: QueryFunction<DB>,
    DB: Database,
{
    fn should_memoize_value(key: &Q::Key) -> bool;

    fn memoized_value_eq(old_value: &Q::Value, new_value: &Q::Value) -> bool;
}

pub enum AlwaysMemoizeValue {}
impl<DB, Q> MemoizationPolicy<DB, Q> for AlwaysMemoizeValue
where
    Q: QueryFunction<DB>,
    Q::Value: Eq,
    DB: Database,
{
    fn should_memoize_value(_key: &Q::Key) -> bool {
        true
    }

    fn memoized_value_eq(old_value: &Q::Value, new_value: &Q::Value) -> bool {
        old_value == new_value
    }
}

pub enum NeverMemoizeValue {}
impl<DB, Q> MemoizationPolicy<DB, Q> for NeverMemoizeValue
where
    Q: QueryFunction<DB>,
    DB: Database,
{
    fn should_memoize_value(_key: &Q::Key) -> bool {
        false
    }

    fn memoized_value_eq(_old_value: &Q::Value, _new_value: &Q::Value) -> bool {
        panic!("cannot reach since we never memoize")
    }
}

impl<DB, Q, MP> Default for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn default() -> Self {
        DerivedStorage {
            database_key_indices: Default::default(),
            slot_map: Default::default(),
            lru_list: Lru::new(),
            policy: PhantomData,
        }
    }
}

impl<DB, Q, MP> DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn database_key_index(&self, db: &DB, key: &Q::Key) -> DatabaseKeyIndex {
        self.database_key_indices.insert(db, key)
    }

    fn slot(&self, db: &DB, key: &Q::Key) -> Arc<Slot<DB, Q, MP>> {
        let database_key_index = self.database_key_index(db, key);

        if let Some(v) = self.slot_map.read().get(&database_key_index) {
            return v.clone();
        }

        let mut write = self.slot_map.write();
        write
            .entry(database_key_index)
            .or_insert_with(|| Arc::new(Slot::new(key.clone())))
            .clone()
    }
}

impl<DB, Q, MP> QueryStorageOps<DB, Q> for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn try_fetch(&self, db: &DB, key: &Q::Key) -> Result<Q::Value, CycleError<DB::DatabaseKey>> {
        let database_key_index = self.database_key_index(db, key);
        let slot = self.slot(db, key);
        let StampedValue {
            value,
            durability,
            changed_at,
        } = slot.read(db)?;

        if let Some(evicted_index) = self.lru_list.record_use(database_key_index) {
            if let Some(evicted_slot) = self.slot_map.read().get(&evicted_index) {
                evicted_slot.evict();
            }
        }

        db.salsa_runtime()
            .report_query_read(slot, durability, changed_at);

        Ok(value)
    }

    fn durability(&self, db: &DB, key: &Q::Key) -> Durability {
        self.slot(db, key).durability(db)
    }

    fn entries<C>(&self, _db: &DB) -> C
    where
        C: std::iter::FromIterator<TableEntry<Q::Key, Q::Value>>,
    {
        let slot_map = self.slot_map.read();
        slot_map
            .values()
            .filter_map(|slot| slot.as_table_entry())
            .collect()
    }
}

impl<DB, Q, MP> QueryStorageMassOps<DB> for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn sweep(&self, db: &DB, strategy: SweepStrategy) {
        let map_read = self.slot_map.read();
        let revision_now = db.salsa_runtime().current_revision();
        for slot in map_read.values() {
            slot.sweep(revision_now, strategy);
        }
    }
}

impl<DB, Q, MP> LruQueryStorageOps for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn set_lru_capacity(&self, new_capacity: usize) {
        self.lru_list.set_lru_capacity(new_capacity);
    }
}

impl<DB, Q, MP> DerivedQueryStorageOps<DB, Q> for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn invalidate(&self, db: &mut DB, key: &Q::Key) {
        let database_key_index = self.database_key_index(db, key);

        db.salsa_runtime_mut().with_incremented_revision(|guard| {
            let map_read = self.slot_map.read();

            if let Some(slot) = map_read.get(&database_key_index) {
                if let Some(durability) = slot.invalidate() {
                    guard.mark_durability_as_changed(durability);
                }
            }
        })
    }
}
