use super::{routes::Ingredients, storage::HasJars, HasJar};

pub trait Jar: Sized {
    fn create_jar<DB>(ingredients: &mut Ingredients<DB>) -> Self
    where
        DB: HasJar<Self> + HasJars;
}
