use crate::testing;

#[must_use]
pub fn substitute_fragment(
    orchestrator: &testing::Orchestrator,
    fragment: &str,
    token: &str,
    live: &[testing::LivePeer],
) -> Option<String> {
    if let Some(name) = fragment.strip_prefix("add-friend:") {
        let found = live.iter().find(|entry| entry.name == name)?;
        let rendered = found
            .peer
            .addrs
            .iter()
            .map(|addr| format!("add-friend:{addr}/p2p/{}", found.peer.id))
            .collect::<Vec<String>>()
            .join(", ");
        return Some(rendered);
    }
    if let Some(name) = fragment.strip_prefix("dial:") {
        let found = live.iter().find(|entry| entry.name == name)?;
        let rendered = found
            .peer
            .addrs
            .iter()
            .map(|addr| format!("{addr}/p2p/{}", found.peer.id))
            .collect::<Vec<String>>()
            .join(" ");
        return Some(format!("dial:{rendered}"));
    }
    if let Some(rest) = fragment.strip_prefix("wait-friend-connected:")
        && let Some((name, deadline)) = rest.rsplit_once('@')
        && let Ok(id) = orchestrator.peer_id_for_name(token, name)
    {
        return Some(format!("wait-friend-connected:{id}@{deadline}"));
    }
    if let Some(rest) = fragment.strip_prefix("wait-friend-disconnected:")
        && let Some((name, deadline)) = rest.rsplit_once('@')
        && let Ok(id) = orchestrator.peer_id_for_name(token, name)
    {
        return Some(format!("wait-friend-disconnected:{id}@{deadline}"));
    }
    Some(fragment.to_string())
}

#[cfg(test)]
mod tests {
    use super::substitute_fragment;
    use crate::identity;
    use crate::testing;

    fn live() -> Vec<testing::LivePeer> {
        vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from("bob-id"),
                addrs: vec![
                    identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001"),
                    identity::PeerAddr::from("/ip4/127.0.0.1/udp/5002/quic-v1"),
                ],
            },
        }]
    }

    fn orchestrator() -> testing::Orchestrator {
        testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        )
    }

    #[test]
    fn test_usage() {
        let rendered = substitute_fragment(&orchestrator(), "add-friend:bob", "token", &live())
            .expect("known peer");
        assert_eq!(
            rendered,
            "add-friend:/ip4/127.0.0.1/tcp/5001/p2p/bob-id, add-friend:/ip4/127.0.0.1/udp/5002/quic-v1/p2p/bob-id"
        );
        let rendered =
            substitute_fragment(&orchestrator(), "dial:bob", "token", &live()).expect("known peer");
        assert_eq!(
            rendered,
            "dial:/ip4/127.0.0.1/tcp/5001/p2p/bob-id /ip4/127.0.0.1/udp/5002/quic-v1/p2p/bob-id"
        );
        assert_eq!(
            substitute_fragment(&orchestrator(), "add-friend:mallory", "token", &live()),
            None
        );
        assert_eq!(
            substitute_fragment(&orchestrator(), "stay", "token", &live()),
            Some("stay".to_string())
        );
    }

    #[test]
    fn waits_resolve_names_to_stable_ids() {
        let rendered = substitute_fragment(
            &orchestrator(),
            "wait-friend-connected:bob@420s",
            "token",
            &live(),
        )
        .expect("stable id");
        let id = orchestrator()
            .peer_id_for_name("token", "bob")
            .expect("identity");
        assert_eq!(rendered, format!("wait-friend-connected:{id}@420s"));
        let rendered = substitute_fragment(
            &orchestrator(),
            "wait-friend-disconnected:bob@60s",
            "token",
            &live(),
        )
        .expect("stable id");
        assert_eq!(rendered, format!("wait-friend-disconnected:{id}@60s"));
        std::fs::remove_file(orchestrator().peer_identity_file("token", "bob")).expect("cleanup");
    }
}
