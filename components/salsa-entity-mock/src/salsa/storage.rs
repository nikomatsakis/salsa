use std::sync::Arc;

use crate::salsa::runtime::Runtime;

use super::routes::Ingredients;
use super::{DatabaseKeyIndex, ParallelDatabase, Revision};

#[allow(dead_code)]
pub struct Storage<DB: HasJars> {
    jars: Arc<DB::Jars>,
    ingredients: Arc<Ingredients<DB>>,
    runtime: Runtime,
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
            jars: Arc::new(jars),
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
            jars: self.jars.clone(),
            ingredients: self.ingredients.clone(),
            runtime: self.runtime.snapshot(),
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn jars(&self) -> (&DB::Jars, &Runtime) {
        (&self.jars, &self.runtime)
    }

    pub fn jars_mut(&mut self) -> (&DB::Jars, &mut Runtime) {
        (&self.jars, &mut self.runtime)
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

pub trait HasJars: HasJarsDyn + Sized {
    type Jars;

    fn jars(&self) -> (&Self::Jars, &Runtime);

    fn jars_mut(&mut self) -> (&Self::Jars, &mut Runtime);

    fn create_jars(ingredients: &mut Ingredients<Self>) -> Self::Jars;
}

pub trait HasJar<J>: HasJarsDyn {
    fn jar(&self) -> (&J, &Runtime);

    fn jar_mut(&mut self) -> (&J, &mut Runtime);
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
}

pub trait IngredientsFor {
    type Jar;
    type Ingredients;

    fn create_ingredients<DB>(ingredients: &mut Ingredients<DB>) -> Self::Ingredients
    where
        DB: HasJars,
        DB: HasJar<Self::Jar>;
}
