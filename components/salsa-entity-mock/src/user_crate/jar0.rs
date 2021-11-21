use crate::salsa::plumbing::IngredientFor;
use crate::salsa::prelude::*;
use crate::salsa::{self, EntityData};

// Source:
//
// #[salsa::jar]
// struct Jar0(Entity0);
//
// trait Jar0Db: salsa::HasJar<Jar0> {}

struct Jar0(<Entity0 as IngredientFor>::Ingredient);

impl salsa::plumbing::HasIngredient<<Entity0 as IngredientFor>::Ingredient> for Jar0 {
    fn ingredient(&self) -> &<Entity0 as IngredientFor>::Ingredient {
        &self.0
    }
}

trait Jar0Db: salsa::HasJar<Jar0> {}

// Source:
//
// #[salsa::Entity(Entity0 in Jar0)]
// struct EntityData0;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct Entity0(salsa::Id);
struct EntityData0;

impl salsa::AsId for Entity0 {}

impl salsa::EntityId for Entity0 {
    type Jar = Jar0;
    type Data = EntityData0;
}

impl salsa::EntityData for EntityData0 {
    type Jar = Jar0;
    type Id = Entity0;
}

impl salsa::plumbing::IngredientFor for Entity0 {
    type Ingredient = salsa::entity::EntityIngredients<Entity0>;
}

// ----

#[allow(dead_code)]
fn foo(db: &dyn Jar0Db) -> &EntityData0 {
    let id = EntityData0.new(db);
    id.data(db)
}
