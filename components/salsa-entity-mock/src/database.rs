use crate::{storage::HasJarsDyn, Event, Runtime};

pub trait Database: HasJarsDyn + AsSalsaDatabase {
    /// This function is invoked at key points in the salsa
    /// runtime. It permits the database to be customized and to
    /// inject logging or other custom behavior.
    fn salsa_event(&self, event_fn: Event) {
        #![allow(unused_variables)]
    }

    fn salsa_runtime(&self) -> &Runtime;
}

pub trait ParallelDatabase: Database {}

pub trait AsSalsaDatabase {
    fn as_salsa_database(&self) -> &dyn Database;
}
