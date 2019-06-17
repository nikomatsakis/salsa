use crate::debug::TableEntry;
use crate::lru::Lru;
use crate::plumbing::CycleDetected;
use crate::plumbing::HasQueryGroup;
use crate::plumbing::LruQueryStorageOps;
use crate::plumbing::QueryFunction;
use crate::plumbing::QueryStorageMassOps;
use crate::plumbing::QueryStorageOps;
use crate::runtime::StampedValue;
use crate::{Database, SweepStrategy};
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

/// "Dependency" queries just track their dependencies and not the
/// actual value (which they produce on demand). This lessens the
/// storage requirements.
pub type VolatileStorage<DB, Q> = DerivedStorage<DB, Q, VolatileValue>;

/// Handles storage where the value is 'derived' by executing a
/// function (in contrast to "inputs").
pub struct DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    lru_list: Lru<Slot<DB, Q, MP>>,
    slot_map: RwLock<FxHashMap<Q::Key, Arc<Slot<DB, Q, MP>>>>,
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

pub trait MemoizationPolicy<DB, Q>
where
    Q: QueryFunction<DB>,
    DB: Database,
{
    fn should_memoize_value(key: &Q::Key) -> bool;

    fn memoized_value_eq(old_value: &Q::Value, new_value: &Q::Value) -> bool;

    fn should_track_inputs(key: &Q::Key) -> bool;
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

    fn should_track_inputs(_key: &Q::Key) -> bool {
        true
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

    fn should_track_inputs(_key: &Q::Key) -> bool {
        true
    }
}

pub enum VolatileValue {}
impl<DB, Q> MemoizationPolicy<DB, Q> for VolatileValue
where
    Q: QueryFunction<DB>,
    DB: Database,
{
    fn should_memoize_value(_key: &Q::Key) -> bool {
        // Why memoize? Well, if the "volatile" value really is
        // constantly changing, we still want to capture its value
        // until the next revision is triggered and ensure it doesn't
        // change -- otherwise the system gets into an inconsistent
        // state where the same query reports back different values.
        true
    }

    fn memoized_value_eq(_old_value: &Q::Value, _new_value: &Q::Value) -> bool {
        false
    }

    fn should_track_inputs(_key: &Q::Key) -> bool {
        false
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
            slot_map: RwLock::new(FxHashMap::default()),
            lru_list: Default::default(),
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
    fn slot(&self, key: &Q::Key) -> Arc<Slot<DB, Q, MP>> {
        if let Some(v) = self.slot_map.read().get(key) {
            return v.clone();
        }

        let mut write = self.slot_map.write();
        write
            .entry(key.clone())
            .or_insert_with(|| Arc::new(Slot::new(key.clone())))
            .clone()
    }

    fn remove_lru(&self) {}
}

impl<DB, Q, MP> QueryStorageOps<DB, Q> for DerivedStorage<DB, Q, MP>
where
    Q: QueryFunction<DB>,
    DB: Database + HasQueryGroup<Q::Group>,
    MP: MemoizationPolicy<DB, Q>,
{
    fn try_fetch(&self, db: &DB, key: &Q::Key) -> Result<Q::Value, CycleDetected> {
        let slot = self.slot(key);
        let StampedValue { value, changed_at } = slot.read(db)?;

        if let Some(evicted) = self.lru_list.record_use(&slot) {
            evicted.evict();
        }

        db.salsa_runtime().report_query_read(slot, changed_at);

        Ok(value)
    }

    fn is_constant(&self, db: &DB, key: &Q::Key) -> bool {
        self.slot(key).is_constant(db)
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
