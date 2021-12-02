use salsa::cycle::CycleRecoveryStrategy;
use salsa::durability::Durability;
use salsa::entity::EntityIngredient;
use salsa::storage::{HasIngredientsFor, IngredientsFor};

// Source:
//
// #[salsa::jar(Jar0Db)]
// pub struct Jar0(Entity0, Ty0, EntityComponent0, my_func);
//
// pub trait Jar0Db: salsa::DbWithJar<Jar0> {}
//
// impl<DB> Jar0Db for DB
// where DB: ?Sized + salsa::DbWithJar<Jar0> {}

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

    fn ingredient_mut(&mut self) -> &mut <Entity0 as IngredientsFor>::Ingredients {
        &mut self.0
    }
}

impl salsa::storage::HasIngredientsFor<Ty0> for Jar0 {
    fn ingredient(&self) -> &<Ty0 as IngredientsFor>::Ingredients {
        &self.1
    }

    fn ingredient_mut(&mut self) -> &mut <Ty0 as IngredientsFor>::Ingredients {
        &mut self.1
    }
}

impl salsa::storage::HasIngredientsFor<EntityComponent0> for Jar0 {
    fn ingredient(&self) -> &<EntityComponent0 as IngredientsFor>::Ingredients {
        &self.2
    }

    fn ingredient_mut(&mut self) -> &mut <EntityComponent0 as IngredientsFor>::Ingredients {
        &mut self.2
    }
}

impl salsa::storage::HasIngredientsFor<my_func> for Jar0 {
    fn ingredient(&self) -> &<my_func as IngredientsFor>::Ingredients {
        &self.3
    }

    fn ingredient_mut(&mut self) -> &mut <my_func as IngredientsFor>::Ingredients {
        &mut self.3
    }
}

impl salsa::jar::Jar for Jar0 {
    type DynDb = dyn Jar0Db;

    fn create_jar<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self
    where
        DB: salsa::storage::HasJars + salsa::storage::DbWithJar<Self>,
        salsa::storage::Storage<DB>: salsa::storage::HasJar<Self>,
    {
        let i0 = <Entity0 as IngredientsFor>::create_ingredients(ingredients);
        let i1 = <Ty0 as IngredientsFor>::create_ingredients(ingredients);
        let i2 = <EntityComponent0 as IngredientsFor>::create_ingredients(ingredients);
        let i3 = <my_func as IngredientsFor>::create_ingredients(ingredients);
        Jar0(i0, i1, i2, i3)
    }
}

// FIXME: 'static
pub trait Jar0Db: salsa::DbWithJar<Jar0> + 'static {}

impl<DB> Jar0Db for DB where DB: ?Sized + salsa::DbWithJar<Jar0> + 'static {}

// Source:
//
// #[salsa::Entity(Entity0 in Jar0)]
// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct EntityData0 {
//    id: u32
// }

mod __entity0 {
    use super::*;
    #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
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
            DB: salsa::storage::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::storage::HasJar::jar(db);
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
            salsa::storage::Storage<DB>: salsa::storage::HasJar<Self::Jar>,
        {
            let index = ingredients.push_mut(
                |storage| {
                    let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
                    <Jar0 as HasIngredientsFor<Self>>::ingredient(jar)
                },
                |storage| {
                    let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar_mut(storage);
                    <Jar0 as HasIngredientsFor<Self>>::ingredient_mut(jar)
                },
            );
            EntityIngredient::new(index)
        }
    }

    #[derive(Eq, PartialEq, Hash, Debug, Clone)]
    pub struct EntityData0 {
        pub(super) id: u32,
    }

    impl EntityData0 {
        pub fn new<DB: ?Sized>(self, db: &DB) -> Entity0
        where
            DB: salsa::storage::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::storage::HasJar::jar(db);
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
// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct TyData0 {
//    id: u32
// }

mod __ty0 {
    use super::*;
    #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
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
            DB: salsa::storage::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::storage::HasJar::jar(db);
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
            salsa::storage::Storage<DB>: salsa::storage::HasJar<Self::Jar>,
        {
            let index = ingredients.push(|storage| {
                let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
                <Jar0 as HasIngredientsFor<Self>>::ingredient(jar)
            });
            salsa::interned::InternedIngredient::new(index)
        }
    }

    #[derive(Eq, PartialEq, Hash, Debug, Clone)]
    pub struct TyData0 {
        pub(super) id: u32,
    }

    impl TyData0 {
        pub fn intern<DB: ?Sized>(self, db: &DB) -> Ty0
        where
            DB: salsa::storage::HasJar<Jar0>,
        {
            let (jar, runtime) = salsa::storage::HasJar::jar(db);
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
    method: salsa::function::FunctionIngredient<EntityComponent0_method>,
}

#[allow(non_camel_case_types)]
struct EntityComponent0_method;

impl salsa::function::Configuration for EntityComponent0_method {
    type Jar = Jar0;

    type Key = Entity0;

    type Value = String;

    const CYCLE_STRATEGY: salsa::cycle::CycleRecoveryStrategy =
        salsa::cycle::CycleRecoveryStrategy::Panic;

    const MEMOIZE_VALUE: bool = true;

    fn should_backdate_value(v1: &Self::Value, v2: &Self::Value) -> bool {
        v1 == v2
    }

    fn execute(db: &salsa::function::DynDb<Self>, key: Self::Key) -> Self::Value {
        trait __Secret__ {
            fn method(self, db: &dyn Jar0Db) -> String;
        }

        impl __Secret__ for Entity0 {
            fn method(self, _db: &dyn Jar0Db) -> String {
                format!("Hello, world")
            }
        }

        <Entity0 as __Secret__>::method(key, db)
    }

    fn recover_from_cycle(
        _db: &salsa::function::DynDb<Self>,
        _cycle: &salsa::Cycle,
        _key: Self::Key,
    ) -> Self::Value {
        panic!()
    }
}

impl IngredientsFor for EntityComponent0 {
    type Jar = Jar0;
    type Ingredients = Self;

    fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
    where
        DB: salsa::storage::HasJars + salsa::DbWithJar<Self::Jar>,
        salsa::storage::Storage<DB>: salsa::storage::HasJar<Self::Jar>,
    {
        let index = ingredients.push(|storage| {
            let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
            let ingredients = <Jar0 as HasIngredientsFor<Self>>::ingredient(jar);
            &ingredients.method
        });
        let method = salsa::function::FunctionIngredient::new(index);

        Self { method }
    }
}

impl Entity0 {
    #[allow(dead_code)]
    fn method(self, db: &dyn Jar0Db) -> String {
        let (jar, _) = salsa::storage::HasJar::jar(db);
        let component: &EntityComponent0 =
            <Jar0 as salsa::storage::HasIngredientsFor<EntityComponent0>>::ingredient(jar);
        component.method.fetch(db, self)
    }

    fn set_method(self, _db: &dyn Jar0Db, _value: String) {
        // TODO:
        //
        // * Check that this entity was created by current query
        //   (either by checking creator of entity or by checking
        //   list of things created by current query, the latter
        //   may be preferred but either should work in principle)
        // * Insert into map
        todo!()
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
    function: salsa::function::FunctionIngredient<my_func>,
}

impl salsa::function::Configuration for my_func {
    type Jar = Jar0;

    type Key = salsa::id::Id;

    type Value = String;

    const CYCLE_STRATEGY: salsa::cycle::CycleRecoveryStrategy = CycleRecoveryStrategy::Panic;

    const MEMOIZE_VALUE: bool = true;

    fn should_backdate_value(old_value: &Self::Value, new_value: &Self::Value) -> bool {
        old_value == new_value
    }

    fn execute(db: &salsa::function::DynDb<Self>, key: Self::Key) -> Self::Value {
        fn __secret__(_db: &dyn Jar0Db, _input1: u32, _input2: u32) -> String {
            format!("Hello, world")
        }

        let (jar, runtime) = salsa::storage::HasJar::jar(db);
        let my_func: &my_func =
            <Jar0 as salsa::storage::HasIngredientsFor<my_func>>::ingredient(jar);
        let key = my_func.intern_map.data(runtime, key).clone();
        __secret__(db, key.0, key.1)
    }

    fn recover_from_cycle(
        _db: &salsa::function::DynDb<Self>,
        _cycle: &salsa::Cycle,
        _key: Self::Key,
    ) -> Self::Value {
        panic!()
    }
}

impl IngredientsFor for my_func {
    type Ingredients = Self;
    type Jar = Jar0;

    fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
    where
        DB: salsa::storage::HasJars + salsa::DbWithJar<Self::Jar>,
        salsa::storage::Storage<DB>: salsa::storage::HasJar<Self::Jar>,
    {
        let index = ingredients.push(|storage| {
            let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
            let ingredients = <Jar0 as HasIngredientsFor<Self::Ingredients>>::ingredient(jar);
            &ingredients.intern_map
        });
        let intern_map = salsa::interned::InternedIngredient::new(index);

        let index = ingredients.push(|storage| {
            let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
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

#[allow(dead_code)]
fn my_func(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
    let (jar, runtime) = salsa::storage::HasJar::jar(db);
    let my_func: &my_func = <Jar0 as salsa::storage::HasIngredientsFor<my_func>>::ingredient(jar);
    let key = my_func.intern_map.intern(runtime, (input1, input2));
    my_func.function.fetch(db, key)
}

impl my_func {
    fn set(db: &mut dyn Jar0Db, input1: u32, input2: u32, value: String) {
        let (jar, runtime) = salsa::storage::HasJar::jar_mut(db);
        let my_func: &mut my_func =
            <Jar0 as salsa::storage::HasIngredientsFor<my_func>>::ingredient_mut(jar);
        let id = my_func.intern_map.intern(runtime, (input1, input2));
        my_func.function.store(id, runtime, value, Durability::LOW);
    }
}

// ----

#[allow(dead_code)]
fn foo(db: &dyn Jar0Db) {
    let id = EntityData0 { id: 0 }.new(db);
    id.data(db);

    let ty = TyData0 { id: 0 }.intern(db);
    ty.data(db);
}
