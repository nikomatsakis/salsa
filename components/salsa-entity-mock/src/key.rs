use std::fmt::Debug;

use crate::{ingredient::Ingredient, Database, Id, IngredientIndex};

/// An integer that uniquely identifies a particular query instance within the
/// database. Used to track dependencies between queries. Fully ordered and
/// equatable but those orderings are arbitrary, and meant to be used only for
/// inserting into maps and the like.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DatabaseKeyIndex {
    pub(crate) ingredient_index: IngredientIndex,
    pub(crate) key_index: Option<Id>,
}

impl DatabaseKeyIndex {
    /// Create a database-key-index for an interning or entity table.
    /// The `key_index` here is always zero, which deliberately corresponds to
    /// no particular id or entry. This is because the data in such tables
    /// remains valid until the table as a whole is reset. Using a single id avoids
    /// creating tons of dependencies in the dependency listings.
    pub(crate) fn for_table(ingredient_index: IngredientIndex) -> Self {
        Self {
            ingredient_index,
            key_index: None,
        }
    }

    pub fn ingredient_index(self) -> IngredientIndex {
        self.ingredient_index
    }

    pub fn debug<DB: ?Sized + Database>(self, db: &DB) -> impl Debug + '_ {
        self // FIXME
    }

    pub(crate) fn is(self, a: ActiveDatabaseKeyIndex) -> bool {
        self.ingredient_index == a.ingredient_index && self.key_index == Some(a.key_index)
    }
}

/// An "active" database key index represents a database key index
/// that is actively executing. In that case, the `key_index` cannot be
/// None.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ActiveDatabaseKeyIndex {
    pub(crate) ingredient_index: IngredientIndex,
    pub(crate) key_index: Id,
}

impl ActiveDatabaseKeyIndex {
    pub fn debug<DB: ?Sized + Database>(self, db: &DB) -> impl Debug + '_ {
        let i: DatabaseKeyIndex = self.into();
        i.debug(db)
    }
}

impl From<ActiveDatabaseKeyIndex> for DatabaseKeyIndex {
    fn from(value: ActiveDatabaseKeyIndex) -> Self {
        Self {
            ingredient_index: value.ingredient_index,
            key_index: Some(value.key_index),
        }
    }
}

impl TryFrom<DatabaseKeyIndex> for ActiveDatabaseKeyIndex {
    type Error = ();

    fn try_from(value: DatabaseKeyIndex) -> Result<Self, Self::Error> {
        let key_index = value.key_index.ok_or(())?;
        Ok(Self {
            ingredient_index: value.ingredient_index,
            key_index,
        })
    }
}
