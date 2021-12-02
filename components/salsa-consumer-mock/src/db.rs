use super::jar0::Jar0;

// User writes:
//
// ```rust
// #[salsa::jars(Jar0, ..., JarN)]
// struct TheDatabase {
//     storage: salsa::Storage<Self>,
// }
//
// impl salsa::Database for TheDatabase {
//     fn salsa_event(&self, event: salsa::Event) {
//         // whatever you want to do here
//     }
// }
//
// impl salsa::ParallelDatabase {
//     fn snapshot(&self) -> Self {
//         Self {
//             storage: self.storage.snapshot(),
//             ...
//         }
//     }
// }
// ```

struct TheDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for TheDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }
}

impl salsa::ParallelDatabase for TheDatabase {}

impl salsa::database::AsSalsaDatabase for TheDatabase {
    fn as_salsa_database(&self) -> &dyn salsa::Database {
        self
    }
}

impl salsa::storage::HasJars for TheDatabase {
    type Jars = (Jar0,);

    fn jars(&self) -> (&Self::Jars, &salsa::Runtime) {
        self.storage.jars()
    }

    fn jars_mut(&mut self) -> (&mut Self::Jars, &mut salsa::Runtime) {
        self.storage.jars_mut()
    }

    fn create_jars(ingredients: &mut salsa::routes::Ingredients<Self>) -> Self::Jars {
        (<Jar0 as salsa::jar::Jar>::create_jar(ingredients),)
    }
}

impl salsa::storage::HasJarsDyn for TheDatabase {
    fn runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }

    fn maybe_changed_after(
        &self,
        input: salsa::DatabaseKeyIndex,
        revision: salsa::Revision,
    ) -> bool {
        let ingredient = self.storage.ingredient(input.ingredient_index());
        ingredient.maybe_changed_after(self, input, revision)
    }

    fn cycle_recovery_strategy(
        &self,
        input: salsa::IngredientIndex,
    ) -> salsa::cycle::CycleRecoveryStrategy {
        todo!()
    }
}

impl salsa::storage::DbWithJar<Jar0> for TheDatabase {
    fn as_jar_db(&self) -> &<Jar0 as salsa::jar::Jar>::DynDb {
        self
    }
}

impl salsa::storage::HasJar<Jar0> for TheDatabase {
    fn jar(&self) -> (&Jar0, &salsa::Runtime) {
        <_ as salsa::storage::HasJar<Jar0>>::jar(&self.storage)
    }

    fn jar_mut(&mut self) -> (&mut Jar0, &mut salsa::Runtime) {
        <_ as salsa::storage::HasJar<Jar0>>::jar_mut(&mut self.storage)
    }
}

impl salsa::storage::HasJar<Jar0> for salsa::Storage<TheDatabase> {
    fn jar(&self) -> (&Jar0, &salsa::Runtime) {
        let (jars, runtime) = self.jars();
        (&jars.0, runtime)
    }

    fn jar_mut(&mut self) -> (&mut Jar0, &mut salsa::Runtime) {
        let (jars, runtime) = self.jars_mut();
        (&mut jars.0, runtime)
    }
}
