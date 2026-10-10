use derive_more::{Deref, From};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, From)]
#[from(forward)]
pub struct GameName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, From)]
#[from(forward)]
pub struct GameToken(pub String);
