use std::num::NonZeroU32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    value: NonZeroU32,
}

impl Id {
    pub fn as_u32(self) -> u32 {
        self.value.get()
    }
}

pub trait AsId: Sized + Copy {
    fn as_id(self) -> Id;
    fn from_id(id: Id) -> Self;
}

impl AsId for Id {
    fn as_id(self) -> Id {
        self
    }

    fn from_id(id: Id) -> Self {
        id
    }
}
