use std::{fmt, hash::Hash, ptr::NonNull};

use crossbeam::atomic::AtomicCell;
use dashmap::mapref::entry::Entry;

use crate::{
    cycle::CycleRecoveryStrategy,
    hash::FxDashMap,
    id::AsId,
    ingredient::{fmt_index, Ingredient, IngredientRequiresReset},
    ingredient_list::IngredientList,
    key::{DatabaseKeyIndex, DependencyIndex},
    runtime::{local_state::QueryOrigin, Runtime},
    salsa_struct::SalsaStructInDb,
    Database, Durability, Event, Id, IngredientIndex, Revision,
};

use self::struct_map::{StructMap, Update};
pub use self::tracked_field::TrackedFieldIngredient;

mod struct_map;
mod tracked_field;

// ANCHOR: Configuration
/// Trait that defines the key properties of a tracked struct.
/// Implemented by the `#[salsa::tracked]` macro when applied
/// to a struct.
pub trait Configuration: Sized {
    /// A (possibly empty) tuple of the fields for this struct.
    type Fields<'db>;

    /// A array of [`Revision`][] values, one per each of the value fields.
    /// When a struct is re-recreated in a new revision, the corresponding
    /// entries for each field are updated to the new revision if their
    /// values have changed (or if the field is marked as `#[no_eq]`).
    type Revisions;

    type Struct<'db>: Copy;

    /// Create an end-user struct from the underlying raw pointer.
    ///
    /// This call is an "end-step" to the tracked struct lookup/creation
    /// process in a given revision: it occurs only when the struct is newly
    /// created or, if a struct is being reused, after we have updated its
    /// fields (or confirmed it is green and no updates are required).
    ///
    /// # Safety
    ///
    /// Requires that `ptr` represents a "confirmed" value in this revision,
    /// which means that it will remain valid and immutable for the remainder of this
    /// revision, represented by the lifetime `'db`.
    unsafe fn struct_from_raw<'db>(ptr: NonNull<ValueStruct<Self>>) -> Self::Struct<'db>;

    /// Deref the struct to yield the underlying value struct.
    /// Since we are still part of the `'db` lifetime in which the struct was created,
    /// this deref is safe, and the value-struct fields are immutable and verified.
    fn deref_struct(s: Self::Struct<'_>) -> &ValueStruct<Self>;

    fn id_fields(fields: &Self::Fields<'_>) -> impl Hash;

    /// Access the revision of a given value field.
    /// `field_index` will be between 0 and the number of value fields.
    fn revision(revisions: &Self::Revisions, field_index: u32) -> Revision;

    /// Create a new value revision array where each element is set to `current_revision`.
    fn new_revisions(current_revision: Revision) -> Self::Revisions;

    /// Update the field data and, if the value has changed,
    /// the appropriate entry in the `revisions` array.
    ///
    /// # Safety
    ///
    /// Requires the same conditions as the `maybe_update`
    /// method on [the `Update` trait](`crate::update::Update`).
    ///
    /// In short, requires that `old_fields` be a pointer into
    /// storage from a previous revision.
    /// It must meet its validity invariant.
    /// Owned content must meet safety invariant.
    /// `*mut` here is not strictly needed;
    /// it is used to signal that the content
    /// is not guaranteed to recursively meet
    /// its safety invariant and
    /// hence this must be dereferenced with caution.
    ///
    /// Ensures that `old_fields` is fully updated and valid
    /// after it returns and that `revisions` has been updated
    /// for any field that changed.
    unsafe fn update_fields<'db>(
        current_revision: Revision,
        revisions: &mut Self::Revisions,
        old_fields: *mut Self::Fields<'db>,
        new_fields: Self::Fields<'db>,
    );
}
// ANCHOR_END: Configuration

pub trait TrackedStructInDb<DB: ?Sized + Database>: SalsaStructInDb<DB> {
    /// Converts the identifier for this tracked struct into a `DatabaseKeyIndex`.
    fn database_key_index(db: &DB, id: Id) -> DatabaseKeyIndex;
}

/// Created for each tracked struct.
/// This ingredient only stores the "id" fields.
/// It is a kind of "dressed up" interner;
/// the active query + values of id fields are hashed to create the tracked struct id.
/// The value fields are stored in [`crate::function::FunctionIngredient`] instances keyed by the tracked struct id.
/// Unlike normal interners, tracked struct indices can be deleted and reused aggressively:
/// when a tracked function re-executes,
/// any tracked structs that it created before but did not create this time can be deleted.
pub struct TrackedStructIngredient<C>
where
    C: Configuration,
{
    /// Our index in the database.
    ingredient_index: IngredientIndex,

    /// Defines the set of live tracked structs.
    /// Entries are added to this map when a new struct is created.
    /// They are removed when that struct is deleted
    /// (i.e., a query completes without having recreated the struct).
    keys: FxDashMap<KeyStruct, Id>,

    /// The number of tracked structs created.
    counter: AtomicCell<u32>,

    /// Map from the [`Id`][] of each struct to its fields/values.
    struct_map: struct_map::StructMap<C>,

    /// A list of each tracked function `f` whose key is this
    /// tracked struct.
    ///
    /// Whenever an instance `i` of this struct is deleted,
    /// each of these functions will be notified
    /// so they can remove any data tied to that instance.
    dependent_fns: IngredientList,

    debug_name: &'static str,
}

/// Defines the identity of a tracked struct.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
struct KeyStruct {
    /// The active query (i.e., tracked function) that created this tracked struct.
    query_key: DatabaseKeyIndex,

    /// The hash of the `#[id]` fields of this struct.
    /// Note that multiple structs may share the same hash.
    data_hash: u64,

    /// The unique disambiguator assigned within the active query
    /// to distinguish distinct tracked structs with the same hash.
    disambiguator: Disambiguator,
}

// ANCHOR: ValueStruct
#[derive(Debug)]
pub struct ValueStruct<C>
where
    C: Configuration,
{
    /// Index of the struct ingredient.
    struct_ingredient_index: IngredientIndex,

    /// The id of this struct in the ingredient.
    id: Id,

    /// The key used to create the id.
    key: KeyStruct,

    /// The durability minimum durability of all inputs consumed
    /// by the creator query prior to creating this tracked struct.
    /// If any of those inputs changes, then the creator query may
    /// create this struct with different values.
    durability: Durability,

    /// The revision when this entity was most recently created.
    /// Typically the current revision.
    /// Used to detect "leaks" outside of the salsa system -- i.e.,
    /// access to tracked structs that have not (yet?) been created in the
    /// current revision. This should be impossible within salsa queries
    /// but it can happen through "leaks" like thread-local data or storing
    /// values outside of the root salsa query.
    created_at: Revision,

    /// Fields of this tracked struct. They can change across revisions,
    /// but they do not change within a particular revision.
    fields: C::Fields<'static>,

    /// The revision information for each field: when did this field last change.
    /// When tracked structs are re-created, this revision may be updated to the
    /// current revision if the value is different.
    revisions: C::Revisions,
}
// ANCHOR_END: ValueStruct

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Disambiguator(pub u32);

impl<C> TrackedStructIngredient<C>
where
    C: Configuration,
{
    /// Convert the fields from a `'db` lifetime to `'static`: used when storing
    /// the data into this ingredient, should never be released outside this type.
    unsafe fn to_static<'db>(&'db self, fields: C::Fields<'db>) -> C::Fields<'static> {
        unsafe { std::mem::transmute(fields) }
    }

    /// Convert from static back to the db lifetime; used when returning data
    /// out from this ingredient.
    unsafe fn to_self_ptr<'db>(&'db self, fields: *mut C::Fields<'static>) -> *mut C::Fields<'db> {
        unsafe { std::mem::transmute(fields) }
    }

    /// Create a tracked struct ingredient. Generated by the `#[tracked]` macro,
    /// not meant to be called directly by end-users.
    pub fn new(index: IngredientIndex, debug_name: &'static str) -> Self {
        Self {
            ingredient_index: index,
            keys: FxDashMap::default(),
            counter: AtomicCell::new(0),
            struct_map: StructMap::new(),
            dependent_fns: IngredientList::new(),
            debug_name,
        }
    }

    /// Creates and returns a new field ingredient for the database.
    /// Invoked by the `#[tracked]` struct and not meant to be called by end-users.
    pub fn new_field_ingredient(
        &self,
        field_ingredient_index: IngredientIndex,
        field_index: u32,
        field_debug_name: &'static str,
    ) -> TrackedFieldIngredient<C> {
        assert_eq!(
            field_ingredient_index.as_u32() - self.ingredient_index.as_u32() - 1,
            field_index,
        );

        TrackedFieldIngredient::<C> {
            ingredient_index: field_ingredient_index,
            field_index,
            struct_map: self.struct_map.view(),
            struct_debug_name: self.debug_name,
            field_debug_name,
        }
    }

    /// Returns the database key index for a tracked struct with the given id.
    pub fn database_key_index(&self, id: Id) -> DatabaseKeyIndex {
        DatabaseKeyIndex {
            ingredient_index: self.ingredient_index,
            key_index: id,
        }
    }

    /// Intern a tracked struct key to get a unique tracked struct id.
    /// Also returns a bool indicating whether this id was newly created or whether it already existed.
    fn intern(&self, key: KeyStruct) -> (Id, bool) {
        let (id, new_id) = if let Some(g) = self.keys.get(&key) {
            (*g.value(), false)
        } else {
            match self.keys.entry(key) {
                Entry::Occupied(o) => (*o.get(), false),
                Entry::Vacant(v) => {
                    let id = Id::from_u32(self.counter.fetch_add(1));
                    v.insert(id);
                    (id, true)
                }
            }
        };

        (id, new_id)
    }

    pub fn new_struct<'db>(
        &'db self,
        runtime: &'db Runtime,
        fields: C::Fields<'db>,
    ) -> C::Struct<'db> {
        let data_hash = crate::hash::hash(&C::id_fields(&fields));

        let (query_key, current_deps, disambiguator) =
            runtime.disambiguate_entity(self.ingredient_index, Revision::start(), data_hash);

        let entity_key = KeyStruct {
            query_key,
            disambiguator,
            data_hash,
        };

        let (id, new_id) = self.intern(entity_key);
        runtime.add_output(self.database_key_index(id).into());

        let current_revision = runtime.current_revision();
        if new_id {
            // This is a new tracked struct, so create an entry in the struct map.

            self.struct_map.insert(
                runtime,
                ValueStruct {
                    id,
                    key: entity_key,
                    struct_ingredient_index: self.ingredient_index,
                    created_at: current_revision,
                    durability: current_deps.durability,
                    fields: unsafe { self.to_static(fields) },
                    revisions: C::new_revisions(current_deps.changed_at),
                },
            )
        } else {
            // The struct already exists in the intern map.
            // Note that we assume there is at most one executing copy of
            // the current query at a time, which implies that the
            // struct must exist in `self.struct_map` already
            // (if the same query could execute twice in parallel,
            // then it would potentially create the same struct twice in parallel,
            // which means the interned key could exist but `struct_map` not yet have
            // been updated).

            match self.struct_map.update(runtime, id) {
                Update::Current(r) => {
                    // All inputs up to this point were previously
                    // observed to be green and this struct was already
                    // verified. Therefore, the durability ought not to have
                    // changed (nor the field values, but the user could've
                    // done something stupid, so we can't *assert* this is true).
                    assert!(C::deref_struct(r).durability == current_deps.durability);

                    r
                }
                Update::Outdated(mut data_ref) => {
                    let data = &mut *data_ref;

                    // SAFETY: We assert that the pointer to `data.revisions`
                    // is a pointer into the database referencing a value
                    // from a previous revision. As such, it continues to meet
                    // its validity invariant and any owned content also continues
                    // to meet its safety invariant.
                    unsafe {
                        C::update_fields(
                            current_revision,
                            &mut data.revisions,
                            self.to_self_ptr(std::ptr::addr_of_mut!(data.fields)),
                            fields,
                        );
                    }
                    if current_deps.durability < data.durability {
                        data.revisions = C::new_revisions(current_revision);
                    }
                    data.durability = current_deps.durability;
                    data.created_at = current_revision;
                    data_ref.freeze()
                }
            }
        }
    }

    /// Given the id of a tracked struct created in this revision,
    /// returns a pointer to the struct.
    ///
    /// # Panics
    ///
    /// If the struct has not been created in this revision.
    pub fn lookup_struct<'db>(&'db self, runtime: &'db Runtime, id: Id) -> C::Struct<'db> {
        self.struct_map.get(runtime, id)
    }

    /// Deletes the given entities. This is used after a query `Q` executes and we can compare
    /// the entities `E_now` that it produced in this revision vs the entities
    /// `E_prev` it produced in the last revision. Any missing entities `E_prev - E_new` can be
    /// deleted.
    ///
    /// # Warning
    ///
    /// Using this method on an entity id that MAY be used in the current revision will lead to
    /// unspecified results (but not UB). See [`InternedIngredient::delete_index`] for more
    /// discussion and important considerations.
    pub(crate) fn delete_entity(&self, db: &dyn crate::Database, id: Id) {
        db.salsa_event(Event {
            runtime_id: db.runtime().id(),
            kind: crate::EventKind::DidDiscard {
                key: self.database_key_index(id),
            },
        });

        if let Some(key) = self.struct_map.delete(id) {
            self.keys.remove(&key);
        }

        for dependent_fn in self.dependent_fns.iter() {
            db.salsa_struct_deleted(dependent_fn, id.as_id());
        }
    }

    /// Adds a dependent function (one keyed by this tracked struct) to our list.
    /// When instances of this struct are deleted, these dependent functions
    /// will be notified.
    pub fn register_dependent_fn(&self, index: IngredientIndex) {
        self.dependent_fns.push(index);
    }
}

impl<DB: ?Sized, C> Ingredient<DB> for TrackedStructIngredient<C>
where
    DB: Database,
    C: Configuration,
{
    fn ingredient_index(&self) -> IngredientIndex {
        self.ingredient_index
    }

    fn maybe_changed_after(&self, _db: &DB, _input: DependencyIndex, _revision: Revision) -> bool {
        false
    }

    fn cycle_recovery_strategy(&self) -> CycleRecoveryStrategy {
        crate::cycle::CycleRecoveryStrategy::Panic
    }

    fn origin(&self, _key_index: crate::Id) -> Option<QueryOrigin> {
        None
    }

    fn mark_validated_output<'db>(
        &'db self,
        db: &'db DB,
        _executor: DatabaseKeyIndex,
        output_key: Option<crate::Id>,
    ) {
        let runtime = db.runtime();
        let output_key = output_key.unwrap();
        self.struct_map.validate(runtime, output_key);
    }

    fn remove_stale_output(
        &self,
        db: &DB,
        _executor: DatabaseKeyIndex,
        stale_output_key: Option<crate::Id>,
    ) {
        // This method is called when, in prior revisions,
        // `executor` creates a tracked struct `salsa_output_key`,
        // but it did not in the current revision.
        // In that case, we can delete `stale_output_key` and any data associated with it.
        self.delete_entity(db.as_salsa_database(), stale_output_key.unwrap());
    }

    fn reset_for_new_revision(&mut self) {
        self.struct_map.drop_deleted_entries();
    }

    fn salsa_struct_deleted(&self, _db: &DB, _id: crate::Id) {
        panic!("unexpected call: interned ingredients do not register for salsa struct deletion events");
    }

    fn fmt_index(&self, index: Option<crate::Id>, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_index(self.debug_name, index, fmt)
    }
}

impl<C> IngredientRequiresReset for TrackedStructIngredient<C>
where
    C: Configuration,
{
    const RESET_ON_NEW_REVISION: bool = true;
}

impl<C> ValueStruct<C>
where
    C: Configuration,
{
    /// Access to this value field.
    /// Note that this function returns the entire tuple of value fields.
    /// The caller is responible for selecting the appropriate element.
    pub fn field<'db>(&'db self, runtime: &'db Runtime, field_index: u32) -> &'db C::Fields<'db> {
        let field_ingredient_index = IngredientIndex::from(
            self.struct_ingredient_index.as_usize() + field_index as usize + 1,
        );
        let changed_at = C::revision(&self.revisions, field_index);

        runtime.report_tracked_read(
            DependencyIndex {
                ingredient_index: field_ingredient_index,
                key_index: Some(self.id.as_id()),
            },
            self.durability,
            changed_at,
        );

        unsafe { self.to_self_ref(&self.fields) }
    }

    unsafe fn to_self_ref<'db>(&'db self, fields: &'db C::Fields<'static>) -> &'db C::Fields<'db> {
        unsafe { std::mem::transmute(fields) }
    }
}

impl<C> AsId for ValueStruct<C>
where
    C: Configuration,
{
    fn as_id(&self) -> Id {
        self.id
    }
}
