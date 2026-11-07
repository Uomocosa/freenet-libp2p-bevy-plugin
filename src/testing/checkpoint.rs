use derive_more::{Deref, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deref, Display)]
pub struct Checkpoint(pub &'static str);

impl Checkpoint {
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::Checkpoint;

    #[test]
    fn test_usage() {
        const DONE: Checkpoint = Checkpoint::new("painting-done");
        assert_eq!(*DONE, "painting-done");
        assert_eq!(DONE, Checkpoint::new("painting-done"));
    }
}
