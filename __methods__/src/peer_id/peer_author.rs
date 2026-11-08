use crate::identity;

#[must_use]
pub fn peer_author(peer: &identity::PeerId) -> u16 {
    let hash = blake3::hash(peer.as_str().as_bytes());
    let mut bytes = hash.as_bytes().iter().copied();
    let high = bytes.next().unwrap_or(0);
    let low = bytes.next().unwrap_or(0);
    u16::from_be_bytes([high, low])
}

#[cfg(test)]
mod tests {
    use super::peer_author;
    use crate::identity;

    #[test]
    fn test_usage() {
        let author = peer_author(&identity::PeerId::from("peer-a"));
        assert_eq!(author, peer_author(&identity::PeerId::from("peer-a")));
        assert_ne!(author, peer_author(&identity::PeerId::from("peer-b")));
    }
}
