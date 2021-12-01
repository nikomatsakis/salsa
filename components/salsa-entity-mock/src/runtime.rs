use std::sync::{atomic::AtomicUsize, Arc};

use crate::{
    durability::Durability,
    hash::{FxIndexMap, FxIndexSet},
    revision::AtomicRevision,
    Cancelled, Cycle, Database, Event, EventKind, Revision,
};

use super::{entity::Disambiguator, DatabaseKeyIndex, IngredientIndex};

mod active_query;
mod dependency_graph;
pub(crate) mod local_state;
mod shared_state;

#[allow(dead_code)]
pub struct Runtime {
    /// Our unique runtime id.
    id: RuntimeId,

    /// Local state that is specific to this runtime (thread).
    local_state: local_state::LocalState,

    /// Shared state that is accessible via all runtimes.
    shared_state: Arc<shared_state::SharedState>,
}

#[derive(Clone, Debug)]
pub(crate) enum WaitResult {
    Completed,
    Panicked,
    Cycle(Cycle),
}

/// A unique identifier for a particular runtime. Each time you create
/// a snapshot, a fresh `RuntimeId` is generated. Once a snapshot is
/// complete, its `RuntimeId` may potentially be re-used.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeId {
    counter: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct StampedValue<V> {
    pub(crate) value: V,
    pub(crate) durability: Durability,
    pub(crate) changed_at: Revision,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            id: RuntimeId { counter: 0 },
            shared_state: Default::default(),
            local_state: Default::default(),
        }
    }
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Runtime")
            .field("id", &self.id())
            .field("shared_state", &self.shared_state)
            .finish()
    }
}

impl Runtime {
    pub(crate) fn id(&self) -> RuntimeId {
        self.id
    }

    pub(crate) fn current_revision(&self) -> Revision {
        self.shared_state.revisions[0].load()
    }

    pub(crate) fn empty_dependencies(&self) -> Arc<[DatabaseKeyIndex]> {
        self.shared_state.empty_dependencies.clone()
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Self {
        todo!()
    }

    pub(crate) fn report_tracked_read(&self, _key_index: DatabaseKeyIndex) {
        todo!()
    }

    /// Reports that the query depends on some state unknown to salsa.
    ///
    /// Queries which report untracked reads will be re-executed in the next
    /// revision.
    pub fn report_untracked_read(&self) {
        self.local_state
            .report_untracked_read(self.current_revision());
    }

    /// Reports that an input with durability `durability` changed.
    /// This will update the 'last changed at' values for every durability
    /// less than or equal to `durability` to the current revision.
    pub(crate) fn report_tracked_write(&mut self, durability: Durability) {
        let new_revision = self.current_revision();
        for rev in &self.shared_state.revisions[1..=durability.index()] {
            rev.store(new_revision);
        }
    }

    /// Called when the active queries creates an index from the
    /// entity table with the index `entity_index`. Has the following effects:
    ///
    /// * Add a query read on `DatabaseKeyIndex::for_table(entity_index)`
    /// * Indentify a unique disambiguator for the hash within the current query,
    ///   adding the hash to the current query's disambiguator table.
    /// * Return that hash + id of the current query.
    pub(crate) fn disambiguate_entity(
        &self,
        entity_index: IngredientIndex,
        _data_hash: u64,
    ) -> (DatabaseKeyIndex, Disambiguator) {
        self.report_tracked_read(DatabaseKeyIndex::for_table(entity_index));
        todo!()
    }

    /// The revision in which values with durability `d` may have last
    /// changed.  For D0, this is just the current revision. But for
    /// higher levels of durability, this value may lag behind the
    /// current revision. If we encounter a value of durability Di,
    /// then, we can check this function to get a "bound" on when the
    /// value may have changed, which allows us to skip walking its
    /// dependencies.
    #[inline]
    pub(crate) fn last_changed_revision(&self, d: Durability) -> Revision {
        self.shared_state.revisions[d.index()].load()
    }

    /// Starts unwinding the stack if the current revision is cancelled.
    ///
    /// This method can be called by query implementations that perform
    /// potentially expensive computations, in order to speed up propagation of
    /// cancellation.
    ///
    /// Cancellation will automatically be triggered by salsa on any query
    /// invocation.
    ///
    /// This method should not be overridden by `Database` implementors. A
    /// `salsa_event` is emitted when this method is called, so that should be
    /// used instead.
    pub(crate) fn unwind_if_revision_cancelled(&self, db: &dyn Database) {
        db.salsa_event(Event {
            runtime_id: self.id(),
            kind: EventKind::WillCheckCancellation,
        });
        if self.shared_state.revision_canceled.load() {
            db.salsa_event(Event {
                runtime_id: self.id(),
                kind: EventKind::WillCheckCancellation,
            });
            self.unwind_cancelled();
        }
    }

    #[cold]
    pub(crate) fn unwind_cancelled(&self) {
        self.report_untracked_read();
        Cancelled::PendingWrite.throw();
    }

    pub(crate) fn set_cancellation_flag(&self) {
        self.shared_state.revision_canceled.store(true);
    }

    /// Increments the "current revision" counter and clears
    /// the cancellation flag.
    ///
    /// This should only be done by the storage when the state is "quiescent".
    pub(crate) fn new_revision(&mut self) -> Revision {
        let r_old = self.current_revision();
        let r_new = r_old.next();
        self.shared_state.revisions[0].store(r_new);
        self.shared_state.revision_canceled.store(false);
        r_new
    }
}
