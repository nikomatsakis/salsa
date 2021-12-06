use super::jar0::Jar0;
use super::lexer::LexerJar;

// User writes:
//
// ```rust
// #[salsa::jars(Jar0, ..., JarN)]
// pub(crate) struct TheDatabase {
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

pub(crate) struct TheDatabase {
    storage: salsa::Storage<Self>,
}

impl Default for TheDatabase {
    fn default() -> Self {
        Self {
            storage: Default::default(),
        }
    }
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
    type Jars = (Jar0, LexerJar);

    fn jars(&self) -> (&Self::Jars, &salsa::Runtime) {
        self.storage.jars()
    }

    fn jars_mut(&mut self) -> (&mut Self::Jars, &mut salsa::Runtime) {
        self.storage.jars_mut()
    }

    fn create_jars(ingredients: &mut salsa::routes::Ingredients<Self>) -> Self::Jars {
        (
            <Jar0 as salsa::jar::Jar>::create_jar(ingredients),
            <LexerJar as salsa::jar::Jar>::create_jar(ingredients),
        )
    }
}

impl salsa::storage::HasJarsDyn for TheDatabase {
    fn runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }

    fn maybe_changed_after(
        &self,
        input: salsa::key::DependencyIndex,
        revision: salsa::Revision,
    ) -> bool {
        let ingredient = self.storage.ingredient(input.ingredient_index());
        ingredient.maybe_changed_after(self, input, revision)
    }

    fn cycle_recovery_strategy(
        &self,
        ingredient_index: salsa::IngredientIndex,
    ) -> salsa::cycle::CycleRecoveryStrategy {
        let ingredient = self.storage.ingredient(ingredient_index);
        ingredient.cycle_recovery_strategy()
    }
}

impl salsa::storage::DbWithJar<Jar0> for TheDatabase {
    fn as_jar_db<'db>(&'db self) -> &'db <Jar0 as salsa::jar::Jar<'db>>::DynDb
    where
        'db: 'db,
    {
        self as &'db <Jar0 as salsa::jar::Jar<'db>>::DynDb
    }
}

impl salsa::storage::HasJar<Jar0> for TheDatabase {
    fn jar(&self) -> (&Jar0, &salsa::Runtime) {
        let (jars, runtime) = self.storage.jars();
        (&jars.0, runtime)
    }

    fn jar_mut(&mut self) -> (&mut Jar0, &mut salsa::Runtime) {
        let (jars, runtime) = self.storage.jars_mut();
        (&mut jars.0, runtime)
    }
}

impl salsa::storage::JarFromJars<Jar0> for TheDatabase {
    fn jar_from_jars<'db>(jars: &Self::Jars) -> &Jar0 {
        &jars.0
    }

    fn jar_from_jars_mut<'db>(jars: &mut Self::Jars) -> &mut Jar0 {
        &mut jars.0
    }
}

impl salsa::storage::DbWithJar<LexerJar> for TheDatabase {
    fn as_jar_db<'db>(&'db self) -> &'db <LexerJar as salsa::jar::Jar<'db>>::DynDb
    where
        'db: 'db,
    {
        self as &'db <LexerJar as salsa::jar::Jar<'db>>::DynDb
    }
}

impl salsa::storage::HasJar<LexerJar> for TheDatabase {
    fn jar(&self) -> (&LexerJar, &salsa::Runtime) {
        let (jars, runtime) = self.storage.jars();
        (&jars.1, runtime)
    }

    fn jar_mut(&mut self) -> (&mut LexerJar, &mut salsa::Runtime) {
        let (jars, runtime) = self.storage.jars_mut();
        (&mut jars.1, runtime)
    }
}

impl salsa::storage::JarFromJars<LexerJar> for TheDatabase {
    fn jar_from_jars<'db>(jars: &Self::Jars) -> &LexerJar {
        &jars.1
    }

    fn jar_from_jars_mut<'db>(jars: &mut Self::Jars) -> &mut LexerJar {
        &mut jars.1
    }
}
