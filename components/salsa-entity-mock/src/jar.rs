use crate::{storage::HasJar, Database, DbWithJar, Storage};

use super::{routes::Ingredients, storage::HasJars};

pub trait Jar<'db>: Sized {
    type DynDb: ?Sized + HasJar<Self> + Database + 'db;

    fn create_jar<DB>(ingredients: &mut Ingredients<DB>) -> Self
    where
        DB: HasJars + DbWithJar<Self>,
        Storage<DB>: HasJar<Self>;
}
