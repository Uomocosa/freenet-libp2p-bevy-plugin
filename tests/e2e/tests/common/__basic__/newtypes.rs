use derive_more::{Deref, Display, From};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From)]
#[from(forward)]
pub struct LobbyToken(pub String);
