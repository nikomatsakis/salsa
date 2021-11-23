use crate::salsa::prelude::*;
use crate::salsa::storage::IngredientFor;
use crate::salsa::{self, EntityData};

// Source:
//
// #[salsa::jar]
// struct Jar0(Entity0, EntityComponent0, my_func);
//
// trait Jar0Db: salsa::HasJar<Jar0> {}

struct Jar0(
    <Entity0 as IngredientFor>::Ingredient,
    <EntityComponent0 as IngredientFor>::Ingredient,
    <my_func as IngredientFor>::Ingredient,
);

impl salsa::storage::HasIngredient<<Entity0 as IngredientFor>::Ingredient> for Jar0 {
    fn ingredient(&self) -> &<Entity0 as IngredientFor>::Ingredient {
        &self.0
    }
}

impl salsa::storage::HasIngredient<<EntityComponent0 as IngredientFor>::Ingredient> for Jar0 {
    fn ingredient(&self) -> &<EntityComponent0 as IngredientFor>::Ingredient {
        &self.1
    }
}

impl salsa::storage::HasIngredient<<my_func as IngredientFor>::Ingredient> for Jar0 {
    fn ingredient(&self) -> &<my_func as IngredientFor>::Ingredient {
        &self.2
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

impl salsa::AsId for Entity0 {
    fn as_id(self) -> salsa::Id {
        self.0
    }

    fn from_id(id: salsa::Id) -> Self {
        Entity0(id)
    }
}

impl salsa::EntityId for Entity0 {
    type Jar = Jar0;
    type Data = EntityData0;
}

impl salsa::EntityData for EntityData0 {
    type Jar = Jar0;
    type Id = Entity0;
}

impl salsa::storage::IngredientFor for Entity0 {
    type Ingredient = salsa::entity::EntityIngredients<Entity0>;
}

// Source:
//
// #[salsa::component(EntityComponent0 in Jar0)]
// impl Entity0 {
//     fn method(self, db: &dyn Jar0Db) -> String {
//         format!("Hello, world")
//     }
// }

struct EntityComponent0 {
    method: salsa::function::FunctionIngredients<(), Entity0, String>,
}

impl IngredientFor for EntityComponent0 {
    type Ingredient = Self;
}

impl Entity0 {
    fn method(self, db: &dyn Jar0Db) -> String {
        trait __Secret__ {
            fn method(self, db: &dyn Jar0Db) -> String;
        }

        impl __Secret__ for Entity0 {
            fn method(self, _db: &dyn Jar0Db) -> String {
                format!("Hello, world")
            }
        }

        let (jar, runtime) = salsa::HasJar::jar(db);
        let component: &EntityComponent0 =
            <Jar0 as salsa::storage::HasIngredient<EntityComponent0>>::ingredient(jar);
        component
            .method
            .fetch(self, runtime, db, <Entity0 as __Secret__>::method)
    }
}

// Source:
//
// #[salsa::storage(in Jar0)]
// fn my_func(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
//     format!("Hello, world")
// }

struct my_func {
    intern_map: salsa::interned::InternedIngredients<salsa::Id, (u32, u32)>,
    function: salsa::function::FunctionIngredients<(), salsa::Id, String>,
}

impl IngredientFor for my_func {
    type Ingredient = Self;
}

fn my_func(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
    fn __secret__(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
        format!("Hello, world")
    }

    let (jar, runtime) = salsa::HasJar::jar(db);
    let my_func: &my_func = <Jar0 as salsa::storage::HasIngredient<my_func>>::ingredient(jar);
    let id = my_func.intern_map.intern(runtime, (input1, input2));
    my_func
        .function
        .fetch(id, runtime, db, |_id, db| __secret__(db, input1, input2))
}

// ----

#[allow(dead_code)]
fn foo(db: &dyn Jar0Db) -> &EntityData0 {
    let id = EntityData0.new(db);
    id.data(db)
}
