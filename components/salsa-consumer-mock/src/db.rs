#[salsa::db(crate::jar0::Jar0, crate::lexer::LexerJar, crate::error::ErrorJar)]
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

impl salsa::ParallelDatabase for TheDatabase {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(TheDatabase {
            storage: self.storage.snapshot(),
        })
    }
}
