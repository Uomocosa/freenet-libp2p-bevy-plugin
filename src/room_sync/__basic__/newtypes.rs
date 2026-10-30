use derive_more::{Deref, From};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Deref, From)]
pub struct ElapsedSinceAppStart(pub std::time::Duration);
