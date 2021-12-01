use crate::Storage;

use super::{routes::Ingredients, storage::HasJars, HasJar};

pub trait Jar: Sized {
    fn create_jar<DB>(ingredients: &mut Ingredients<DB>) -> Self
    where
        DB: HasJars,
        Storage<DB>: HasJar<Self>;
}
