use std::sync::Arc;

use crate::salsa::{plumbing::HasJars, runtime::Runtime, ParallelDatabase};

#[allow(dead_code)]
pub struct Storage<DB: HasJars> {
    jars: Arc<DB::Jars>,
    runtime: Runtime,
}

impl<DB> Default for Storage<DB>
where
    DB: HasJars,
{
    fn default() -> Self {
        Self {
            jars: Arc::new(DB::empty_jars()),
            runtime: Runtime::default(),
        }
    }
}

impl<DB> Storage<DB>
where
    DB: HasJars,
{
    #[allow(dead_code)]
    fn snapshot(&self) -> Storage<DB>
    where
        DB: ParallelDatabase,
    {
        Self {
            jars: self.jars.clone(),
            runtime: self.runtime.snapshot(),
        }
    }
}
