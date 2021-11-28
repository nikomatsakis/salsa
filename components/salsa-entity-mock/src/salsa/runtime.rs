use super::{entity::Disambiguator, DatabaseKeyIndex, IngredientIndex};

#[allow(dead_code)]
pub struct Runtime {}

impl Default for Runtime {
    fn default() -> Self {
        Self {}
    }
}

impl Runtime {
    #[allow(dead_code)]
    pub fn snapshot(&self) -> Self {
        todo!()
    }

    pub(crate) fn add_query_read(&self, _key_index: DatabaseKeyIndex) {
        todo!()
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
        self.add_query_read(DatabaseKeyIndex::for_table(entity_index));
        todo!()
    }
}
