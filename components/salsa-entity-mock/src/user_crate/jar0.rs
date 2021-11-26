use crate::salsa::entity::EntityIngredient;
use crate::salsa::storage::{HasIngredientsFor, IngredientsFor};
use crate::salsa::{self};

// Source:
//
// #[salsa::jar]
// struct Jar0(Entity0, Ty0, EntityComponent0, my_func);
//
// trait Jar0Db: salsa::HasJar<Jar0> {}

pub struct Jar0(
    <Entity0 as IngredientsFor>::Ingredients,
    <Ty0 as IngredientsFor>::Ingredients,
    <EntityComponent0 as IngredientsFor>::Ingredients,
    <my_func as IngredientsFor>::Ingredients,
);

impl salsa::storage::HasIngredientsFor<Entity0> for Jar0 {
    fn ingredient(&self) -> &<Entity0 as IngredientsFor>::Ingredients {
        &self.0
    }
}

impl salsa::storage::HasIngredientsFor<Ty0> for Jar0 {
    fn ingredient(&self) -> &<Ty0 as IngredientsFor>::Ingredients {
        &self.1
    }
}

impl salsa::storage::HasIngredientsFor<EntityComponent0> for Jar0 {
    fn ingredient(&self) -> &<EntityComponent0 as IngredientsFor>::Ingredients {
        &self.2
    }
}

impl salsa::storage::HasIngredientsFor<my_func> for Jar0 {
    fn ingredient(&self) -> &<my_func as IngredientsFor>::Ingredients {
        &self.3
    }
}

impl salsa::jar::Jar for Jar0 {
    fn create_jar<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self
    where
        DB: salsa::HasJar<Self> + salsa::storage::HasJars,
    {
        let i0 = <Entity0 as IngredientsFor>::create_ingredients(ingredients);
        let i1 = <Ty0 as IngredientsFor>::create_ingredients(ingredients);
        let i2 = <EntityComponent0 as IngredientsFor>::create_ingredients(ingredients);
        let i3 = <my_func as IngredientsFor>::create_ingredients(ingredients);
        Jar0(i0, i1, i2, i3)
    }
}

trait Jar0Db: salsa::HasJar<Jar0> {}

// Source:
//
// #[salsa::Entity(Entity0 in Jar0)]
// struct EntityData0 {
//    id: u32
// }

mod __entity0 {
    use super::*;
    #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct Entity0(salsa::Id);

    impl salsa::AsId for Entity0 {
        fn as_id(self) -> salsa::Id {
            self.0
        }

        fn from_id(id: salsa::Id) -> Self {
            Entity0(id)
        }
    }

    impl Entity0 {
        pub fn data<DB: ?Sized>(self, db: &DB) -> &EntityData0
        where
            DB: salsa::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::HasJar::jar(db);
            return helper(jar, runtime, self);

            fn helper<'j>(
                jar: &'j Jar0,
                runtime: &'j salsa::Runtime,
                id: Entity0,
            ) -> &'j EntityData0 {
                let ingredients = <Jar0 as HasIngredientsFor<Entity0>>::ingredient(jar);
                ingredients.entity_data(runtime, id)
            }
        }
    }

    impl salsa::storage::IngredientsFor for Entity0 {
        type Jar = Jar0;
        type Ingredients = salsa::entity::EntityIngredient<Entity0, EntityData0>;

        fn create_ingredients<DB>(
            ingredients: &mut salsa::routes::Ingredients<DB>,
        ) -> Self::Ingredients
        where
            DB: salsa::storage::HasJars,
            DB: salsa::HasJar<Self::Jar>,
        {
            let index = ingredients.push(|db| {
                let (jar, _) = <DB as salsa::HasJar<Self::Jar>>::jar(db);
                <Jar0 as HasIngredientsFor<Self>>::ingredient(jar)
            });
            EntityIngredient::new(index)
        }
    }

    pub struct EntityData0 {
        pub(super) id: u32,
    }

    impl EntityData0 {
        pub fn new<DB: ?Sized>(self, db: &DB) -> Entity0
        where
            DB: salsa::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::HasJar::jar(db);
            return helper(jar, runtime, self);

            fn helper(jar: &Jar0, runtime: &salsa::Runtime, data: EntityData0) -> Entity0 {
                // just to reduce monomorphization cost
                let ingredients = <Jar0 as HasIngredientsFor<Entity0>>::ingredient(jar);
                ingredients.new_entity(runtime, data)
            }
        }
    }
}
pub(self) use self::__entity0::Entity0;
pub(self) use self::__entity0::EntityData0;

// Source:
//
// #[salsa::interned(Ty0 in Jar0)]
// struct TyData0 {
//    id: u32
// }

mod __ty0 {
    use super::*;
    #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct Ty0(salsa::Id);

    impl salsa::AsId for Ty0 {
        fn as_id(self) -> salsa::Id {
            self.0
        }

        fn from_id(id: salsa::Id) -> Self {
            Ty0(id)
        }
    }

    impl Ty0 {
        pub fn data<DB: ?Sized>(self, db: &DB) -> &TyData0
        where
            DB: salsa::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::HasJar::jar(db);
            return helper(jar, runtime, self);

            fn helper<'j>(jar: &'j Jar0, runtime: &'j salsa::Runtime, id: Ty0) -> &'j TyData0 {
                let ingredients = <Jar0 as HasIngredientsFor<Ty0>>::ingredient(jar);
                ingredients.data(runtime, id)
            }
        }
    }

    impl salsa::storage::IngredientsFor for Ty0 {
        type Jar = Jar0;
        type Ingredients = salsa::interned::InternedIngredient<Ty0, TyData0>;

        fn create_ingredients<DB>(
            ingredients: &mut salsa::routes::Ingredients<DB>,
        ) -> Self::Ingredients
        where
            DB: salsa::storage::HasJars,
            DB: salsa::HasJar<Self::Jar>,
        {
            let index = ingredients.push(|db| {
                let (jar, _) = <DB as salsa::HasJar<Self::Jar>>::jar(db);
                <Jar0 as HasIngredientsFor<Self>>::ingredient(jar)
            });
            salsa::interned::InternedIngredient::new(index)
        }
    }

    pub struct TyData0 {
        pub(super) id: u32,
    }

    impl TyData0 {
        pub fn intern<DB: ?Sized>(self, db: &DB) -> Ty0
        where
            DB: salsa::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::HasJar::jar(db);
            return helper(jar, runtime, self);

            fn helper(jar: &Jar0, runtime: &salsa::Runtime, data: TyData0) -> Ty0 {
                // just to reduce monomorphization cost
                let ingredients = <Jar0 as HasIngredientsFor<Ty0>>::ingredient(jar);
                ingredients.intern(runtime, data)
            }
        }
    }
}
pub(self) use self::__ty0::Ty0;
pub(self) use self::__ty0::TyData0;

// Source:
//
// #[salsa::component(EntityComponent0 in Jar0)]
// impl Entity0 {
//     fn method(self, db: &dyn Jar0Db) -> String {
//         format!("Hello, world")
//     }
// }

pub struct EntityComponent0 {
    method: salsa::function::FunctionIngredient<(), Entity0, String>,
}

impl IngredientsFor for EntityComponent0 {
    type Jar = Jar0;
    type Ingredients = Self;

    fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
    where
        DB: salsa::storage::HasJars,
        DB: salsa::HasJar<Self::Jar>,
    {
        let index = ingredients.push(|db| {
            let (jar, _) = <DB as salsa::HasJar<Self::Jar>>::jar(db);
            let ingredients = <Jar0 as HasIngredientsFor<Self>>::ingredient(jar);
            &ingredients.method
        });
        let method = salsa::function::FunctionIngredient::new(index);

        Self { method }
    }
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
            <Jar0 as salsa::storage::HasIngredientsFor<EntityComponent0>>::ingredient(jar);
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

#[allow(non_camel_case_types)]
pub struct my_func {
    intern_map: salsa::interned::InternedIngredient<salsa::Id, (u32, u32)>,
    function: salsa::function::FunctionIngredient<(), salsa::Id, String>,
}

impl IngredientsFor for my_func {
    type Ingredients = Self;
    type Jar = Jar0;

    fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
    where
        DB: salsa::storage::HasJars,
        DB: salsa::HasJar<Self::Jar>,
    {
        let index = ingredients.push(|db| {
            let (jar, _) = <DB as salsa::HasJar<Self::Jar>>::jar(db);
            let ingredients = <Jar0 as HasIngredientsFor<Self::Ingredients>>::ingredient(jar);
            &ingredients.intern_map
        });
        let intern_map = salsa::interned::InternedIngredient::new(index);

        let index = ingredients.push(|db| {
            let (jar, _) = <DB as salsa::HasJar<Self::Jar>>::jar(db);
            let ingredients = <Jar0 as HasIngredientsFor<Self::Ingredients>>::ingredient(jar);
            &ingredients.function
        });
        let function = salsa::function::FunctionIngredient::new(index);

        my_func {
            intern_map,
            function,
        }
    }
}

fn my_func(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
    fn __secret__(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
        format!("Hello, world")
    }

    let (jar, runtime) = salsa::HasJar::jar(db);
    let my_func: &my_func = <Jar0 as salsa::storage::HasIngredientsFor<my_func>>::ingredient(jar);
    let id = my_func.intern_map.intern(runtime, (input1, input2));
    my_func
        .function
        .fetch(id, runtime, db, |_id, db| __secret__(db, input1, input2))
}

// ----

#[allow(dead_code)]
fn foo(db: &dyn Jar0Db) {
    let id = EntityData0 { id: 0 }.new(db);
    id.data(db);

    let ty = TyData0 { id: 0 }.intern(db);
    ty.data(db);
}
