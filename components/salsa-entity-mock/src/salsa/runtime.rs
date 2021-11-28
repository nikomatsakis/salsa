use super::DatabaseKeyIndex;

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
}
