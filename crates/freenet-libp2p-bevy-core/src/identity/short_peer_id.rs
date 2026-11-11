use derive_more::Deref;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct ShortPeerId(pub String);

impl From<&identity::PeerId> for ShortPeerId {
    fn from(id: &identity::PeerId) -> Self {
        let chars: Vec<char> = id.as_str().chars().collect();
        if chars.len() <= 14 {
            return Self(id.as_str().to_string());
        }
        let split = chars.len().saturating_sub(4);
        let (head_chars, tail_chars) = chars.split_at(split);
        let head: String = head_chars.iter().take(7).collect();
        let tail: String = tail_chars.iter().collect();
        Self(format!("{head}...{tail}"))
    }
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
