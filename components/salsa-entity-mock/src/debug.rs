use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

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

pub struct DebugWith<'me, Db: ?Sized> {
    value: Box<dyn DebugWithDb<Db> + 'me>,
    db: &'me Db,
}

impl<D: ?Sized> std::fmt::Debug for DebugWith<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DebugWithDb::fmt(&*self.value, f, self.db)
    }
}

impl<Db: ?Sized, T> DebugWithDb<Db> for &T
where
    T: DebugWithDb<Db> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        T::fmt(self, f, db)
    }
}

impl<Db: ?Sized, T> DebugWithDb<Db> for Box<T>
where
    T: DebugWithDb<Db> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        T::fmt(self, f, db)
    }
}
impl<Db: ?Sized, T> DebugWithDb<Db> for Rc<T>
where
    T: DebugWithDb<Db> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        T::fmt(self, f, db)
    }
}

impl<Db: ?Sized, T> DebugWithDb<Db> for Arc<T>
where
    T: DebugWithDb<Db> + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        T::fmt(self, f, db)
    }
}

impl<Db: ?Sized, T> DebugWithDb<Db> for Vec<T>
where
    T: DebugWithDb<Db>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let elements = self.iter().map(|e| e.debug(db));
        f.debug_list().entries(elements).finish()
    }
}

impl<Db: ?Sized, K, V, S> DebugWithDb<Db> for HashMap<K, V, S>
where
    K: DebugWithDb<Db>,
    V: DebugWithDb<Db>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let elements = self.iter().map(|(k, v)| (k.debug(db), v.debug(db)));
        f.debug_map().entries(elements).finish()
    }
}

impl<Db: ?Sized, A, B> DebugWithDb<Db> for (A, B)
where
    A: DebugWithDb<Db>,
    B: DebugWithDb<Db>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.0.debug(db))
            .field(&self.1.debug(db))
            .finish()
    }
}

impl<Db: ?Sized, A, B, C> DebugWithDb<Db> for (A, B, C)
where
    A: DebugWithDb<Db>,
    B: DebugWithDb<Db>,
    C: DebugWithDb<Db>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.0.debug(db))
            .field(&self.1.debug(db))
            .field(&self.2.debug(db))
            .finish()
    }
}

impl<Db: ?Sized, V, S> DebugWithDb<Db> for HashSet<V, S>
where
    V: DebugWithDb<Db>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let elements = self.iter().map(|e| e.debug(db));
        f.debug_list().entries(elements).finish()
    }
}
