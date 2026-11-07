use crate::testing;

pub struct PeerBuilder {
    pub(crate) name: String,
    pub(crate) steps: Vec<testing::StoredStep>,
    pub(crate) starts_after: Option<testing::Checkpoint>,
    pub(crate) token_override: Option<String>,
}

impl PeerBuilder {
    pub(crate) const fn new(name: String) -> Self {
        Self {
            name,
            steps: Vec::new(),
            starts_after: None,
            token_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PeerBuilder;

    #[test]
    fn test_usage() {
        let peer = PeerBuilder::new("painter".to_string());
        assert_eq!(peer.name, "painter");
        assert!(peer.starts_after.is_none());
        assert_eq!(peer.steps.len(), 0);
    }
}
