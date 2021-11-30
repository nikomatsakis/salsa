use crate::{
    durability::Durability,
    entity::Disambiguator,
    hash::{FxIndexMap, FxIndexSet},
    Cycle, DatabaseKeyIndex, Revision,
};

use super::local_state::{QueryInputs, QueryRevisions};

#[derive(Debug)]
pub(super) struct ActiveQuery {
    /// What query is executing
    pub(super) database_key_index: DatabaseKeyIndex,

    /// Minimum durability of inputs observed so far.
    pub(super) durability: Durability,

    /// Maximum revision of all inputs observed. If we observe an
    /// untracked read, this will be set to the most recent revision.
    pub(super) changed_at: Revision,

    /// Set of subqueries that were accessed thus far, or `None` if
    /// there was an untracked the read.
    pub(super) dependencies: Option<FxIndexSet<DatabaseKeyIndex>>,

    /// Stores the entire cycle, if one is found and this query is part of it.
    pub(super) cycle: Option<Cycle>,

    /// When new entities are created, their data is hashed, and the resulting
    /// hash is added to this map. If it is not present, then the disambiguator is 0.
    /// Otherwise it is 1 more than the current value (which is incremented).
    pub(super) disambiguator_map: FxIndexMap<u64, Disambiguator>,
}

impl ActiveQuery {
    pub(super) fn new(database_key_index: DatabaseKeyIndex) -> Self {
        ActiveQuery {
            database_key_index,
            durability: Durability::MAX,
            changed_at: Revision::start(),
            dependencies: Some(FxIndexSet::default()),
            cycle: None,
            disambiguator_map: Default::default(),
        }
    }

    pub(super) fn add_read(
        &mut self,
        input: DatabaseKeyIndex,
        durability: Durability,
        revision: Revision,
    ) {
        if let Some(set) = &mut self.dependencies {
            set.insert(input);
        }

        self.durability = self.durability.min(durability);
        self.changed_at = self.changed_at.max(revision);
    }

    pub(super) fn add_untracked_read(&mut self, changed_at: Revision) {
        self.dependencies = None;
        self.durability = Durability::LOW;
        self.changed_at = changed_at;
    }

    pub(super) fn add_synthetic_read(&mut self, durability: Durability, revision: Revision) {
        self.dependencies = None;
        self.durability = self.durability.min(durability);
        self.changed_at = self.changed_at.max(revision);
    }

    pub(crate) fn revisions(&self) -> QueryRevisions {
        let inputs = match &self.dependencies {
            None => QueryInputs::Untracked,

            Some(dependencies) => {
                if dependencies.is_empty() {
                    QueryInputs::NoInputs
                } else {
                    QueryInputs::Tracked {
                        inputs: dependencies.iter().copied().collect(),
                    }
                }
            }
        };

        QueryRevisions {
            changed_at: self.changed_at,
            inputs,
            durability: self.durability,
        }
    }

    /// Adds any dependencies from `other` into `self`.
    /// Used during cycle recovery, see [`Runtime::create_cycle_error`].
    pub(super) fn add_from(&mut self, other: &ActiveQuery) {
        self.changed_at = self.changed_at.max(other.changed_at);
        self.durability = self.durability.min(other.durability);
        if let Some(other_dependencies) = &other.dependencies {
            if let Some(my_dependencies) = &mut self.dependencies {
                my_dependencies.extend(other_dependencies.iter().copied());
            }
        } else {
            self.dependencies = None;
        }
    }

    /// Removes the participants in `cycle` from my dependencies.
    /// Used during cycle recovery, see [`Runtime::create_cycle_error`].
    pub(super) fn remove_cycle_participants(&mut self, cycle: &Cycle) {
        if let Some(my_dependencies) = &mut self.dependencies {
            for p in cycle.participant_keys() {
                my_dependencies.remove(&p);
            }
        }
    }

    /// Copy the changed-at, durability, and dependencies from `cycle_query`.
    /// Used during cycle recovery, see [`Runtime::create_cycle_error`].
    pub(crate) fn take_inputs_from(&mut self, cycle_query: &ActiveQuery) {
        self.changed_at = cycle_query.changed_at;
        self.durability = cycle_query.durability;
        self.dependencies = cycle_query.dependencies.clone();
    }

    pub(super) fn disambiguate(&mut self, hash: u64) -> Disambiguator {
        let disambiguator = self
            .disambiguator_map
            .entry(hash)
            .or_insert(Disambiguator(0));
        let result = *disambiguator;
        disambiguator.0 += 1;
        result
    }
}
