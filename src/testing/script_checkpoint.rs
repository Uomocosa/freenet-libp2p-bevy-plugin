use derive_more::{Deref, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deref, Display)]
pub struct ScriptCheckpoint(pub &'static str);

impl ScriptCheckpoint {
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptCheckpoint;

    #[test]
    fn test_usage() {
        const DONE: ScriptCheckpoint = ScriptCheckpoint::new("painting-done");
        assert_eq!(*DONE, "painting-done");
        assert_eq!(DONE, ScriptCheckpoint::new("painting-done"));
    }
}
