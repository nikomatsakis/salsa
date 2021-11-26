use super::{ingredient::Ingredient, storage::HasJars};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct IngredientIndex(u32);

impl IngredientIndex {
    fn from(v: usize) -> Self {
        assert!(v < (std::u32::MAX as usize));
        Self(v as u32)
    }

    fn as_usize(self) -> usize {
        self.0 as usize
    }
}

pub struct Ingredients<DB: HasJars> {
    jars: Vec<Box<dyn Fn(&DB) -> &dyn Ingredient>>,
}

impl<DB: HasJars> Ingredients<DB> {
    pub(super) fn new() -> Self {
        Ingredients { jars: vec![] }
    }

    pub fn push(&mut self, route: impl (Fn(&DB) -> &dyn Ingredient) + 'static) -> IngredientIndex {
        let len = self.jars.len();
        self.jars.push(Box::new(route));
        IngredientIndex::from(len)
    }

    pub fn route(&self, index: IngredientIndex) -> &dyn Fn(&DB) -> &dyn Ingredient {
        &self.jars[index.as_usize()]
    }
}
