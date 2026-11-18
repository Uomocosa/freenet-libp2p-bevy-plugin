use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Deref)]
pub struct PopupGap(pub f32);
