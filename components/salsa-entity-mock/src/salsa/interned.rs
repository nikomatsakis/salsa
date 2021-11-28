use crossbeam::atomic::AtomicCell;
use std::hash::Hash;

use crate::salsa;
use crate::salsa::id::AsId;
use crate::salsa::runtime::Runtime;

use super::hash::FxDashMap;
use super::ingredient::Ingredient;
use super::routes::IngredientIndex;
use super::{DatabaseKeyIndex, Revision};

pub trait InternedId: AsId + Eq + Hash {}
impl<T: AsId + Eq + Hash> InternedId for T {}

pub trait InternedData: Sized + Eq + Hash + Clone {}
impl<T: Eq + Hash + Clone> InternedData for T {}

#[allow(dead_code)]
pub struct InternedIngredient<Id: InternedId, Data: InternedData> {
    ingredient_index: IngredientIndex,
    key_map: FxDashMap<Data, Id>,
    value_map: FxDashMap<Id, Box<Data>>,
    counter: AtomicCell<u32>,
    reset_at: Revision,
}

impl<Id, Data> InternedIngredient<Id, Data>
where
    Id: InternedId,
    Data: InternedData,
{
    pub fn new(ingredient_index: IngredientIndex) -> Self {
        Self {
            ingredient_index,
            key_map: Default::default(),
            value_map: Default::default(),
            counter: Default::default(),
            reset_at: Revision::start(),
        }
    }

    #[allow(dead_code)]
    pub fn intern(&self, runtime: &Runtime, data: Data) -> Id {
        runtime.add_query_read(DatabaseKeyIndex::for_table(self.ingredient_index));

        if let Some(id) = self.key_map.get(&data) {
            return *id;
        }

        loop {
            let next_id = self.counter.fetch_add(1);
            let next_id = Id::from_id(salsa::id::Id::from_u32(next_id));
            match self.value_map.entry(next_id) {
                // If we already have an entry with this id...
                dashmap::mapref::entry::Entry::Occupied(entry) => continue,

                // Otherwise...
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    self.key_map.insert(data.clone(), next_id);
                    entry.insert(Box::new(data));
                    return next_id;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self, revision: Revision) {
        assert!(revision > self.reset_at);
        self.reset_at = revision;
        self.key_map.clear();
        self.value_map.clear();
    }

    #[allow(dead_code)]
    pub fn data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        runtime.add_query_read(DatabaseKeyIndex {
            ingredient_index: self.ingredient_index,
            key_index: 0,
        });

        let data = self.value_map.get(&id).unwrap();

        // Unsafety clause:
        //
        // * Values are only removed or altered when we have `&mut self`
        unsafe { transmute_lifetime(self, &**data) }
    }

    /// Get the ingredient index for this table.
    pub(crate) fn ingredient_index(&self) -> IngredientIndex {
        self.ingredient_index
    }
}

// Returns `u` but with the lifetime of `t`.
//
// Safe if you know that data at `u` will remain shared
// until the reference `t` expires.
unsafe fn transmute_lifetime<'t, 'u, T, U>(_t: &'t T, u: &'u U) -> &'t U {
    std::mem::transmute(u)
}

impl<Id, Data> Ingredient for InternedIngredient<Id, Data>
where
    Id: InternedId,
    Data: InternedData,
{
    fn maybe_changed_after(
        &self,
        _input: super::DatabaseKeyIndex,
        revision: super::Revision,
    ) -> bool {
        revision < self.reset_at
    }
}
