use crate::salsa::id::AsId;
use crate::salsa::plumbing::{HasIngredient, HasJar};
use crate::salsa::runtime::Runtime;

pub trait EntityId: AsId {
    type Jar: HasIngredient<EntityIngredients<Self>>;
    type Data: EntityData<Id = Self, Jar = Self::Jar>;

    fn data<'db, DB>(self, db: &'db DB) -> &'db Self::Data
    where
        DB: ?Sized + HasJar<Self::Jar>,
        Self: 'db, // XXX don't love this, but then again, Self is truly going to be 'static
    {
        let (jar, runtime) = HasJar::jar(db);
        HasIngredient::ingredient(jar).data(runtime, self)
    }
}

pub trait EntityData: Sized {
    type Jar: HasIngredient<EntityIngredients<Self::Id>>;
    type Id: EntityId<Data = Self, Jar = Self::Jar>;

    fn new<DB>(self, db: &DB) -> Self::Id
    where
        DB: ?Sized + HasJar<Self::Jar>,
    {
        let (jar, runtime) = HasJar::jar(db);
        HasIngredient::ingredient(jar).new(runtime, self)
    }
}

pub trait EntityJar<Id: EntityId> {
    fn ingredients(&self) -> &EntityIngredients<Id>;
}

#[allow(dead_code)]
pub struct EntityIngredients<Id: EntityId> {
    phantom: std::marker::PhantomData<Id>,
}

impl<Id: EntityId> EntityIngredients<Id> {
    #[allow(dead_code)]
    pub fn new(&self, runtime: &Runtime, data: Id::Data) -> Id {
        let _ = (runtime, data);
        panic!()
    }

    #[allow(dead_code)]
    pub fn data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Id::Data {
        let _ = (runtime, id);
        panic!()
    }
}
