use crate::{storage::HasJar, Database, DbWithJar, Storage};

use super::{routes::Ingredients, storage::HasJars};

pub trait Jar: Sized {
    type DynDb: ?Sized + HasJar<Self> + Database;

    fn create_jar<DB>(ingredients: &mut Ingredients<DB>) -> Self
    where
        DB: HasJars + DbWithJar<Self>,
        Storage<DB>: HasJar<Self>;
}
