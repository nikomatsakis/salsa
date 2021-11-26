use super::{DatabaseKeyIndex, Revision};

pub trait Ingredient {
    fn maybe_changed_after(&self, input: DatabaseKeyIndex, revision: Revision) -> bool;
}
