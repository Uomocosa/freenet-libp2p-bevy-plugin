use crate::p2p;

#[must_use]
pub fn is_hole_punched(kinds: &[p2p::ConnectionKind]) -> bool {
    let mut seen_relayed = false;
    let mut hole_punched = false;
    for kind in kinds {
        match kind {
            p2p::ConnectionKind::Relayed => seen_relayed = true,
            p2p::ConnectionKind::Tcp | p2p::ConnectionKind::Quic if seen_relayed => {
                hole_punched = true;
            }
            p2p::ConnectionKind::Tcp | p2p::ConnectionKind::Quic => {}
        }
    }
    hole_punched
}

#[cfg(test)]
mod tests {
    use super::is_hole_punched;
    use crate::p2p;

    #[test]
    fn test_usage() {
        assert!(is_hole_punched(&[
            p2p::ConnectionKind::Relayed,
            p2p::ConnectionKind::Tcp
        ]));
        assert!(!is_hole_punched(&[p2p::ConnectionKind::Tcp]));
        assert!(!is_hole_punched(&[p2p::ConnectionKind::Relayed]));
    }
}
