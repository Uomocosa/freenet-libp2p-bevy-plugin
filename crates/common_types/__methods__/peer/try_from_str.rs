use crate::Peer;
use crate::PeerAddr;
use crate::PeerError;
use crate::PeerId;

pub fn try_from_str(text: &str) -> Result<Peer, PeerError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(PeerError::Empty);
    }
    let addr: multiaddr::Multiaddr = trimmed.parse()?;
    let ending = addr.iter().last();
    let multiaddr::Protocol::P2p(id) = ending.as_ref().ok_or(PeerError::MissingPeerId)? else {
        return Err(PeerError::MissingPeerId);
    };
    Ok(Peer {
        id: PeerId::from(id.to_string()),
        addrs: vec![PeerAddr::from(trimmed)],
    })
}

#[cfg(test)]
mod tests {
    use super::try_from_str;
    use crate::PeerError;

    #[test]
    fn test_usage() {
        assert!(matches!(try_from_str(""), Err(PeerError::Empty)));
    }
}
