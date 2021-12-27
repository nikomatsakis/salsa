pub trait DebugWithDb<Db: ?Sized> {
    fn debug<'me>(&'me self, db: &'me Db) -> DebugWith<'me, Db> {
        DebugWith {
            value: Box::new(self),
            db,
        }
    }

    fn into_debug<'me>(self, db: &'me Db) -> DebugWith<'me, Db>
    where
        Self: Sized + 'me,
    {
        DebugWith {
            value: Box::new(self),
            db,
        }
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result;
}

impl<Db: ?Sized, T> DebugWithDb<Db> for &T
where
    T: DebugWithDb<Db> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        T::fmt(self, f, db)
    }
}

pub struct DebugWith<'me, Db: ?Sized> {
    value: Box<dyn DebugWithDb<Db> + 'me>,
    db: &'me Db,
}

impl<D: ?Sized> std::fmt::Debug for DebugWith<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DebugWithDb::fmt(&*self.value, f, self.db)
    }
}
