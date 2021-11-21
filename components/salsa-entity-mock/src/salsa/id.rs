use std::num::NonZeroU32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    value: NonZeroU32,
}

pub trait AsId: Sized + Copy {}
