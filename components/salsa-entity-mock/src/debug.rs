pub trait DebugWithDb<Db: ?Sized> {
    fn debug<'me>(&'me self, db: &'me Db) -> DebugWith<'me, Self, Db> {
        DebugWith { value: self, db }
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result;
}

pub struct DebugWith<'me, Value: ?Sized, Db: ?Sized> {
    value: &'me Value,
    db: &'me Db,
}

impl<V: ?Sized, D: ?Sized> std::fmt::Debug for DebugWith<'_, V, D>
where
    V: DebugWithDb<D>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DebugWithDb::fmt(self.value, f, self.db)
    }
}
