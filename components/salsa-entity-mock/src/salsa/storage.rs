use std::sync::Arc;

use crate::salsa::runtime::Runtime;

use super::ParallelDatabase;

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

pub trait HasJars: HasJarsDyn {
    type Jars;

    fn jars(&self) -> &Self::Jars;

    fn jars_mut(&mut self) -> (&Self::Jars, &mut Runtime);

    // Avoid relying on impl Default for tuple,
    // because I don't think that works for arbitrary arity.
    fn empty_jars() -> Self::Jars;
}

pub trait HasJar<J>: HasJarsDyn {
    fn jar(&self) -> (&J, &Runtime);

    fn jar_mut(&mut self) -> (&J, &mut Runtime);
}

// Dyn friendly subset of HasJars
pub trait HasJarsDyn {
    fn runtime(&self) -> &Runtime;
}

pub trait HasIngredient<I> {
    fn ingredient(&self) -> &I;
}

pub trait IngredientFor {
    type Ingredient;
}
