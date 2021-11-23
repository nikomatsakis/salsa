use crate::salsa::id::AsId;
use crate::salsa::runtime::Runtime;
use crate::salsa::storage::HasJar;

pub trait InternedId: AsId {
    type Jar: InternedJar<Self>;
    type Data;

    fn data<'db, DB>(self, db: &'db DB) -> &'db Self::Data
    where
        DB: ?Sized + HasJar<Self::Jar>,
        Self: 'db, // XXX don't love this, but then again, Self is truly going to be 'static
    {
        let (jar, runtime) = HasJar::jar(db);
        InternedJar::ingredients(jar).data(runtime, self)
    }
}

pub trait InternedData: Sized {
    type Jar: InternedJar<Self::Id>;
    type Id: InternedId<Jar = Self::Jar, Data = Self>;

    fn intern<DB>(self, db: &DB) -> Self::Id
    where
        DB: ?Sized + HasJar<Self::Jar>,
    {
        let (jar, runtime) = HasJar::jar(db);
        InternedJar::ingredients(jar).intern(runtime, self)
    }
}

pub trait InternedJar<Id: InternedId> {
    fn ingredients(&self) -> &InternedIngredients<Id, Id::Data>;
}

#[allow(dead_code)]
pub struct InternedIngredients<Id: AsId, Data> {
    phantom: std::marker::PhantomData<(Id, Data)>,
}

impl<Id: AsId, Data> InternedIngredients<Id, Data> {
    #[allow(dead_code)]
    pub fn intern(&self, runtime: &Runtime, data: Data) -> Id {
        let _ = (runtime, data);
        panic!()
    }

    #[allow(dead_code)]
    pub fn data<'db>(&'db self, runtime: &'db Runtime, id: Id) -> &'db Data {
        let _ = (runtime, id);
        panic!()
    }
}
