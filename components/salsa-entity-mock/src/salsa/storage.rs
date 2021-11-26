use std::sync::Arc;

use crate::salsa::ingredient::Ingredient;
use crate::salsa::runtime::Runtime;

use super::routes::{IngredientIndex, Ingredients};
use super::ParallelDatabase;

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

    pub fn route(&self, index: IngredientIndex) -> &dyn Fn(&DB) -> &dyn Ingredient {
        self.ingredients.route(index)
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
