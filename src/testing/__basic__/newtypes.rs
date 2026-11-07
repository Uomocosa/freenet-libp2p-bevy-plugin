use derive_more::{Deref, From};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct MemberCount(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct SenderCount(pub u32);
