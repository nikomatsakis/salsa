use crate::salsa;

pub trait HasJars: HasJarsDyn {
    type Jars;

    fn jars(&self) -> &Self::Jars;

    fn jars_mut(&mut self) -> (&Self::Jars, &mut salsa::Runtime);

    // Avoid relying on impl Default for tuple,
    // because I don't think that works for arbitrary arity.
    fn empty_jars() -> Self::Jars;
}

pub trait HasJar<J>: HasJarsDyn {
    fn jar(&self) -> (&J, &salsa::Runtime);

    fn jar_mut(&mut self) -> (&J, &mut salsa::Runtime);
}

// Dyn friendly subset of HasJars
pub trait HasJarsDyn {
    fn runtime(&self) -> &salsa::Runtime;
}

pub trait HasIngredient<I> {
    fn ingredient(&self) -> &I;
}

pub trait IngredientFor {
    type Ingredient;
}
