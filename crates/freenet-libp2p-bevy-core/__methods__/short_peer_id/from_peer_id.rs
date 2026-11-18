use crate::identity;

pub fn from_peer_id(id: &identity::PeerId) -> identity::ShortPeerId {
    let chars: Vec<char> = id.as_str().chars().collect();
    if chars.len() <= 14 {
        return identity::ShortPeerId(id.as_str().to_string());
    }
    let split = chars.len().saturating_sub(4);
    let (head_chars, tail_chars) = chars.split_at(split);
    let head: String = head_chars.iter().take(7).collect();
    let tail: String = tail_chars.iter().collect();
    identity::ShortPeerId(format!("{head}...{tail}"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::from_peer_id;
    use crate::identity;

    #[test]
    fn test_usage() {
        let id = identity::PeerId::from("alice");
        assert_eq!(from_peer_id(&id).as_str(), "alice");
    }

    #[rstest]
    #[case::short_id("alice", "alice")]
    #[case::full_peer_id(
        "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
        "12D3Koo...99RC"
    )]
    fn short_peer_id_truncates(#[case] raw: &str, #[case] expected: &str) {
        let id = identity::PeerId::from(raw);
        assert_eq!(from_peer_id(&id).as_str(), expected);
    }
}
