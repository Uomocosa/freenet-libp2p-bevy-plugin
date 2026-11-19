use derive_more::{Deref, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deref, Display)]
pub struct ScriptCheckpoint(pub &'static str);

impl ScriptCheckpoint {
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}
