use std::{fmt, sync::Arc};

use parking_lot::Condvar;

use crate::cycle::CycleRecoveryStrategy;
use crate::ingredient::Ingredient;
use crate::jar::Jar;
use crate::key::DependencyIndex;
use crate::runtime::local_state::QueryOrigin;
use crate::runtime::Runtime;
use crate::{Database, DatabaseKeyIndex, Id, IngredientIndex};

use super::routes::Routes;
use super::{ParallelDatabase, Revision};

/// The "storage" struct stores all the data for the jars.
/// It is shared between the main database and any active snapshots.
pub struct Storage<DB: HasJars> {
    /// Data shared across all databases. This contains the ingredients needed by each jar.
    /// See the ["jars and ingredients" chapter](https://salsa-rs.github.io/salsa/plumbing/jars_and_ingredients.html)
    /// for more detailed description.
    shared: Shared<DB>,

    /// The "ingredients" structure stores the information about how to find each ingredient in the database.
    /// It allows us to take the [`IngredientIndex`] assigned to a particular ingredient
    /// and get back a [`dyn Ingredient`][`Ingredient`] for the struct that stores its data.
    ///
    /// This is kept separate from `shared` so that we can clone it and retain `&`-access even when we have `&mut` access to `shared`.
    routes: Arc<Routes<DB>>,

    /// The runtime for this particular salsa database handle.
    /// Each handle gets its own runtime, but the runtimes have shared state between them.
    runtime: Runtime,
}

/// Data shared between all threads.
/// This is where the actual data for tracked functions, structs, inputs, etc lives,
/// along with some coordination variables between treads.
struct Shared<DB: HasJars> {
    /// Contains the data for each jar in the database.
    /// Each jar stores its own structs in there that ultimately contain ingredients
    /// (types that implement the [`Ingredient`] trait, like [`crate::function::FunctionIngredient`]).
    ///
    /// Even though these jars are stored in an `Arc`, we sometimes get mutable access to them
    /// by using `Arc::get_mut`. This is only possible when all parallel snapshots have been dropped.
    jars: Option<Arc<DB::Jars>>,

    /// Conditional variable that is used to coordinate cancellation.
    /// When the main thread writes to the database, it blocks until each of the snapshots can be cancelled.
    cvar: Arc<Condvar>,

    /// Mutex that is used to protect the `jars` field when waiting for snapshots to be dropped.
    noti_lock: Arc<parking_lot::Mutex<()>>,
}

// ANCHOR: default
impl<DB> Default for Storage<DB>
where
    DB: HasJars,
{
    fn default() -> Self {
        let mut routes = Routes::new();
        let jars = DB::create_jars(&mut routes);
        Self {
            shared: Shared {
                jars: Some(Arc::from(jars)),
                cvar: Arc::new(Default::default()),
                noti_lock: Arc::new(parking_lot::Mutex::new(())),
            },
            routes: Arc::new(routes),
            runtime: Runtime::default(),
        }
    }
}
// ANCHOR_END: default

impl<DB> Storage<DB>
where
    DB: HasJars,
{
    pub fn snapshot(&self) -> Storage<DB>
    where
        DB: ParallelDatabase,
    {
        Self {
            shared: self.shared.clone(),
            routes: self.routes.clone(),
            runtime: self.runtime.snapshot(),
        }
    }

    pub fn jars(&self) -> (&DB::Jars, &Runtime) {
        (self.shared.jars.as_ref().unwrap(), &self.runtime)
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        self.jars_mut().1
    }

    // ANCHOR: jars_mut
    /// Gets mutable access to the jars. This will trigger a new revision
    /// and it will also cancel any ongoing work in the current revision.
    /// Any actual writes that occur to data in a jar should use
    /// [`Runtime::report_tracked_write`].
    pub fn jars_mut(&mut self) -> (&mut DB::Jars, &mut Runtime) {
        // Wait for all snapshots to be dropped.
        self.cancel_other_workers();

        // Increment revision counter.
        self.runtime.new_revision();

        // Acquire `&mut` access to `self.shared` -- this is only possible because
        // the snapshots have all been dropped, so we hold the only handle to the `Arc`.
        let jars = Arc::get_mut(self.shared.jars.as_mut().unwrap()).unwrap();

        // Inform other ingredients that a new revision has begun.
        // This gives them a chance to free resources that were being held until the next revision.
        let routes = self.routes.clone();
        for route in routes.reset_routes() {
            route(jars).reset_for_new_revision();
        }

        // Return mut ref to jars + runtime.
        (jars, &mut self.runtime)
    }
    // ANCHOR_END: jars_mut

    // ANCHOR: cancel_other_workers
    /// Sets cancellation flag and blocks until all other workers with access
    /// to this storage have completed.
    ///
    /// This could deadlock if there is a single worker with two handles to the
    /// same database!
    fn cancel_other_workers(&mut self) {
        loop {
            self.runtime.set_cancellation_flag();

            // Acquire lock before we check if we have unique access to the jars.
            // If we do not yet have unique access, we will go to sleep and wait for
            // the snapshots to be dropped, which will signal the cond var associated
            // with this lock.
            //
            // NB: We have to acquire the lock first to ensure that we can check for
            // unique access and go to sleep waiting on the condvar atomically,
            // as described in PR #474.
            let mut guard = self.shared.noti_lock.lock();
            // If we have unique access to the jars, we are done.
            if Arc::get_mut(self.shared.jars.as_mut().unwrap()).is_some() {
                return;
            }

            // Otherwise, wait until some other storage entities have dropped.
            //
            // The cvar `self.shared.cvar` is notified by the `Drop` impl.
            self.shared.cvar.wait(&mut guard);
        }
    }
    // ANCHOR_END: cancel_other_workers

    pub fn ingredient(&self, ingredient_index: IngredientIndex) -> &dyn Ingredient<DB> {
        let route = self.routes.route(ingredient_index);
        route(self.shared.jars.as_ref().unwrap())
    }
}

impl<DB> Clone for Shared<DB>
where
    DB: HasJars,
{
    fn clone(&self) -> Self {
        Self {
            jars: self.jars.clone(),
            cvar: self.cvar.clone(),
            noti_lock: self.noti_lock.clone(),
        }
    }
}

impl<DB> Drop for Storage<DB>
where
    DB: HasJars,
{
    fn drop(&mut self) {
        // Drop the Arc reference before the cvar is notified,
        // since other threads are sleeping, waiting for it to reach 1.
        let _guard = self.shared.noti_lock.lock();
        drop(self.shared.jars.take());
        self.shared.cvar.notify_all();
    }
}

pub trait HasJars: HasJarsDyn + Sized {
    type Jars;

    fn jars(&self) -> (&Self::Jars, &Runtime);

    /// Gets mutable access to the jars. This will trigger a new revision
    /// and it will also cancel any ongoing work in the current revision.
    fn jars_mut(&mut self) -> (&mut Self::Jars, &mut Runtime);

    fn create_jars(routes: &mut Routes<Self>) -> Box<Self::Jars>;
}

pub trait DbWithJar<J>: HasJar<J> + Database {
    fn as_jar_db<'db>(&'db self) -> &<J as Jar<'db>>::DynDb
    where
        J: Jar<'db>;
}

pub trait JarFromJars<J>: HasJars {
    fn jar_from_jars(jars: &Self::Jars) -> &J;

    fn jar_from_jars_mut(jars: &mut Self::Jars) -> &mut J;
}

pub trait HasJar<J> {
    fn jar(&self) -> (&J, &Runtime);

    fn jar_mut(&mut self) -> (&mut J, &mut Runtime);
}

// ANCHOR: HasJarsDyn
/// Dyn friendly subset of HasJars
pub trait HasJarsDyn {
    fn runtime(&self) -> &Runtime;

    fn runtime_mut(&mut self) -> &mut Runtime;

    fn maybe_changed_after(&self, input: DependencyIndex, revision: Revision) -> bool;

    fn cycle_recovery_strategy(&self, input: IngredientIndex) -> CycleRecoveryStrategy;

    fn origin(&self, input: DatabaseKeyIndex) -> Option<QueryOrigin>;

    fn mark_validated_output(&self, executor: DatabaseKeyIndex, output: DependencyIndex);

    /// Invoked when `executor` used to output `stale_output` but no longer does.
    /// This method routes that into a call to the [`remove_stale_output`](`crate::ingredient::Ingredient::remove_stale_output`)
    /// method on the ingredient for `stale_output`.
    fn remove_stale_output(&self, executor: DatabaseKeyIndex, stale_output: DependencyIndex);

    /// Informs `ingredient` that the salsa struct with id `id` has been deleted.
    /// This means that `id` will not be used in this revision and hence
    /// any memoized values keyed by that struct can be discarded.
    ///
    /// In order to receive this callback, `ingredient` must have registered itself
    /// as a dependent function using
    /// [`SalsaStructInDb::register_dependent_fn`](`crate::salsa_struct::SalsaStructInDb::register_dependent_fn`).
    fn salsa_struct_deleted(&self, ingredient: IngredientIndex, id: Id);

    fn fmt_index(&self, index: DependencyIndex, fmt: &mut fmt::Formatter<'_>) -> fmt::Result;
}
// ANCHOR_END: HasJarsDyn

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

    fn create_ingredients<DB>(routes: &mut Routes<DB>) -> Self::Ingredients
    where
        DB: DbWithJar<Self::Jar> + JarFromJars<Self::Jar>;
}
