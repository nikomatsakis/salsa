use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

use crate::runtime::Runtime;

use super::routes::Ingredients;
use super::{DatabaseKeyIndex, ParallelDatabase, Revision};

#[allow(dead_code)]
pub struct Storage<DB: HasJars> {
    shared: Arc<Shared<DB>>,
    ingredients: Arc<Ingredients<DB>>,
    runtime: Runtime,
}

struct Shared<DB: HasJars> {
    jars: DB::Jars,
    cvar: Condvar,
}

trait Jar {}

impl<DB> Default for Storage<DB>
where
    DB: HasJars,
{
    fn default() -> Self {
        let mut ingredients = Ingredients::new();
        let jars = DB::create_jars(&mut ingredients);
        Self {
            shared: Arc::new(Shared {
                jars,
                cvar: Default::default(),
            }),
            ingredients: Arc::new(ingredients),
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
            shared: self.shared.clone(),
            ingredients: self.ingredients.clone(),
            runtime: self.runtime.snapshot(),
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn jars(&self) -> (&DB::Jars, &Runtime) {
        (&self.shared.jars, &self.runtime)
    }

    /// Gets mutable access to the jars. This will trigger a new revision
    /// and it will also cancel any ongoing work in the current revision.
    /// Any actual writes that occur to data in a jar should use
    /// [`Runtime::report_tracked_write`].
    pub fn jars_mut(&mut self) -> (&mut DB::Jars, &mut Runtime) {
        // Have to wait for anyone else using shared to drop:
        loop {
            self.runtime.set_cancellation_flag();

            // If we have unique access to the jars, we are done.
            //
            // NB: We don't use `if let Some(shared) = Arc::get_mut(...)` here
            // because of rust-lang/rust#54663.
            if Arc::get_mut(&mut self.shared).is_some() {
                let shared = Arc::get_mut(&mut self.shared).unwrap();
                self.runtime.new_revision();
                return (&mut shared.jars, &mut self.runtime);
            }

            // Otherwise, wait until some other storage entites have dropped.
            // We create a mutex here because the cvar api requires it, but we
            // don't really need one as the data being protected is actually
            // the jars above.
            let mutex = parking_lot::Mutex::new(());
            let mut guard = mutex.lock();
            self.shared.cvar.wait(&mut guard);
        }
    }

    pub fn maybe_changed_after(
        &self,
        db: &DB,
        input: DatabaseKeyIndex,
        revision: Revision,
    ) -> bool {
        let route = self.ingredients.route(input.ingredient_index);
        let ingredient = route(db);
        ingredient.maybe_changed_after(input, revision)
    }
}

impl<DB> Drop for Shared<DB>
where
    DB: HasJars,
{
    fn drop(&mut self) {
        self.cvar.notify_all();
    }
}

pub trait HasJars: HasJarsDyn + Sized {
    type Jars;

    fn jars(&self) -> (&Self::Jars, &Runtime);

    /// Gets mutable access to the jars. This will trigger a new revision
    /// and it will also cancel any ongoing work in the current revision.
    fn jars_mut(&mut self) -> (&mut Self::Jars, &mut Runtime);

    fn create_jars(ingredients: &mut Ingredients<Self>) -> Self::Jars;
}

pub trait HasJar<J>: HasJarsDyn {
    fn jar(&self) -> (&J, &Runtime);

    fn jar_mut(&mut self) -> (&mut J, &mut Runtime);
}

// Dyn friendly subset of HasJars
pub trait HasJarsDyn {
    fn runtime(&self) -> &Runtime;

    fn maybe_changed_after(&self, input: DatabaseKeyIndex, revision: Revision) -> bool;
}

pub trait HasIngredientsFor<I>
where
    I: IngredientsFor,
{
    fn ingredient(&self) -> &I::Ingredients;
    fn ingredient_mut(&mut self) -> &mut I::Ingredients;
}

pub trait IngredientsFor {
    type Jar;
    type Ingredients;

    fn create_ingredients<DB>(ingredients: &mut Ingredients<DB>) -> Self::Ingredients
    where
        DB: HasJars,
        DB: HasJar<Self::Jar>;
}
