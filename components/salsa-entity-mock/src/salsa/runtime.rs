use super::{storage::HasJars, DatabaseKeyIndex, Revision};

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
}
