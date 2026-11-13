use derive_more::Deref;

use atomic_delegate_macros::atomic_delegates;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct ShortPeerId(pub String);

#[atomic_delegates]
impl From<&identity::PeerId> for ShortPeerId {
    fn from(id: &identity::PeerId) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::ShortPeerId;
    use crate::identity;

    #[test]
    fn test_usage() {
        let id = identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC");
        assert_eq!(ShortPeerId::from(&id).as_str(), "12D3Koo...99RC");
    }

    #[test]
    fn short_ids_stay_whole() {
        let id = identity::PeerId::from("alice");
        assert_eq!(ShortPeerId::from(&id).as_str(), "alice");
    }

    #[test]
    fn ids_no_longer_than_the_short_form_stay_whole() {
        let id = identity::PeerId::from("twelve-chars");
        assert_eq!(ShortPeerId::from(&id).as_str(), "twelve-chars");
    }
}
