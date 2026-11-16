use crate::identity;
use crate::testing;

pub fn peer_from_statuses(
    name: &str,
    prefix: &str,
    statuses: &[testing::harness::BoardStatus],
) -> Option<identity::Peer> {
    let mut indexed: Vec<(usize, &testing::harness::BoardStatus)> = statuses
        .iter()
        .filter_map(|status| {
            status
                .context
                .strip_prefix(&format!("{prefix}/peer/{name}/"))?
                .parse::<usize>()
                .ok()
                .map(|index| (index, status))
        })
        .collect();
    indexed.sort_by_key(|(index, _)| *index);
    let mut id: Option<identity::PeerId> = None;
    let mut addrs: Vec<identity::PeerAddr> = Vec::new();
    for (_, status) in indexed {
        let description = status.description.as_deref().unwrap_or("");
        if let Ok(peer) = identity::Peer::try_from(description) {
            let suffix = format!("/p2p/{}", peer.id);
            if id.is_none() {
                id = Some(peer.id.clone());
            }
            for addr in peer.addrs {
                let text = addr.to_string();
                let bare = text.strip_suffix(suffix.as_str()).unwrap_or(text.as_str());
                addrs.push(identity::PeerAddr::from(bare));
            }
        } else if let Some(raw) = description.strip_prefix("peer-id:")
            && !raw.is_empty()
            && id.is_none()
        {
            id = Some(identity::PeerId::from(raw));
        }
    }
    id.map(|id| identity::Peer { id, addrs })
}

#[cfg(test)]
mod tests {
    use super::peer_from_statuses;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let peer = identity::Peer {
            id: identity::PeerId::from(format!("12D3Koo{}", "a".repeat(45))),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        };
        let descriptions = testing::harness::GitHubCheckpointBoard::peer_descriptions(&peer);
        let status: testing::harness::BoardStatus = serde_json::from_value(serde_json::json!({
            "context": "run-1/peer/alice/0",
            "description": descriptions[0],
            "state": "success",
        }))
        .expect("decode");
        let found = peer_from_statuses("alice", "run-1", &[status]).expect("peer");
        assert_eq!(found, peer);
        assert!(peer_from_statuses("mallory", "run-1", &[]).is_none());
    }
}
