use std::marker::PhantomData;

use crate::{ingredient::Ingredient, zalsa::IngredientIndex, Database, DatabaseKeyIndex, Id};

use super::{Configuration, Value};

/// Created for each tracked struct.
///
/// This ingredient only stores the "id" fields.
/// It is a kind of "dressed up" interner;
/// the active query + values of id fields are hashed to create the tracked struct id.
/// The value fields are stored in [`crate::function::FunctionIngredient`] instances keyed by the tracked struct id.
/// Unlike normal interners, tracked struct indices can be deleted and reused aggressively:
/// when a tracked function re-executes,
/// any tracked structs that it created before but did not create this time can be deleted.
pub struct FieldIngredientImpl<C>
where
    C: Configuration,
{
    /// Index of this ingredient in the database (used to construct database-ids, etc).
    ingredient_index: IngredientIndex,
    field_index: usize,
    phantom: PhantomData<fn() -> Value<C>>,
}

impl<C> FieldIngredientImpl<C>
where
    C: Configuration,
{
    pub(super) fn new(struct_index: IngredientIndex, field_index: usize) -> Self {
        Self {
            ingredient_index: struct_index.successor(field_index),
            field_index,
            phantom: PhantomData,
        }
    }
}

impl<C> Ingredient for FieldIngredientImpl<C>
where
    C: Configuration,
{
    fn ingredient_index(&self) -> IngredientIndex {
        self.ingredient_index
    }

    fn cycle_recovery_strategy(&self) -> crate::cycle::CycleRecoveryStrategy {
        crate::cycle::CycleRecoveryStrategy::Panic
    }

    fn maybe_changed_after<'db>(
        &'db self,
        db: &'db dyn Database,
        input: Option<Id>,
        revision: crate::Revision,
    ) -> bool {
        let zalsa = db.zalsa();
        let id = input.unwrap();
        let data = <super::IngredientImpl<C>>::data(zalsa.table(), id);
        let field_changed_at = data.revisions[self.field_index];
        field_changed_at > revision
    }

    fn origin(
        &self,
        _db: &dyn Database,
        _key_index: crate::Id,
    ) -> Option<crate::zalsa_local::QueryOrigin> {
        None
    }

    fn mark_validated_output(
        &self,
        _db: &dyn Database,
        _executor: crate::DatabaseKeyIndex,
        _output_key: Option<crate::Id>,
    ) {
        // instances of this ingredient are not recorded as outputs
        unreachable!(
            "unexpected call to `mark_validated_output` on `{}`",
            std::any::type_name::<Self>()
        )
    }

    fn discard_stale_output(
        &self,
        _db: &dyn Database,
        _executor: DatabaseKeyIndex,
        _stale_output_key: Option<crate::Id>,
    ) {
        // instances of this ingredient are not recorded as outputs
        unreachable!(
            "unexpected call to `discard_stale_output` on `{}`",
            std::any::type_name::<Self>()
        )
    }

    fn requires_reset_for_new_revision(&self) -> bool {
        false
    }

    fn reset_for_new_revision(&mut self) {
        // `requires_reset_for_new_revision` returns false, this should not be called
        unreachable!(
            "unexpected call to `reset_for_new_revision` on `{}`",
            std::any::type_name::<Self>()
        )
    }

    fn fmt_index(
        &self,
        index: Option<crate::Id>,
        fmt: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            fmt,
            "{}.{}({:?})",
            C::DEBUG_NAME,
            C::FIELD_DEBUG_NAMES[self.field_index],
            index.unwrap()
        )
    }

    fn debug_name(&self) -> &'static str {
        C::FIELD_DEBUG_NAMES[self.field_index]
    }

    fn accumulated<'db>(
        &'db self,
        _db: &'db dyn Database,
        _key_index: Id,
    ) -> Option<&'db crate::accumulator::accumulated_map::AccumulatedMap> {
        None
    }
}

impl<C> std::fmt::Debug for FieldIngredientImpl<C>
where
    C: Configuration,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("ingredient_index", &self.ingredient_index)
            .field("field_index", &self.field_index)
            .finish()
    }
}
