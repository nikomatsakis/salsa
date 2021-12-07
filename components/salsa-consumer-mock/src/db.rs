#[salsa::db(super::jar0::Jar0, super::lexer::LexerJar)]
pub(crate) struct TheDatabase {
    storage: salsa::Storage<Self>,
}

impl Default for TheDatabase {
    fn default() -> Self {
        Self {
            storage: Default::default(),
        }
    }
}

impl salsa::Database for TheDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }
}

impl salsa::ParallelDatabase for TheDatabase {}
