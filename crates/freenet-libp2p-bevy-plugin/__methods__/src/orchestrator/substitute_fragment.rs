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
        if found.peer.addrs.is_empty() {
            return None;
        }
        let suffix = format!("/p2p/{}", found.peer.id);
        let rendered = found
            .peer
            .addrs
            .iter()
            .map(|addr| {
                let text = addr.to_string();
                if text.ends_with(suffix.as_str()) {
                    format!("add-friend:{text}")
                } else {
                    format!("add-friend:{text}{suffix}")
                }
            })
            .collect::<Vec<String>>()
            .join(", ");
        return Some(rendered);
    }
    if let Some(name) = fragment.strip_prefix("dial:") {
        let found = live.iter().find(|entry| entry.name == name)?;
        if found.peer.addrs.is_empty() {
            return None;
        }
        let suffix = format!("/p2p/{}", found.peer.id);
        let rendered = found
            .peer
            .addrs
            .iter()
            .map(|addr| {
                let text = addr.to_string();
                if text.ends_with(suffix.as_str()) {
                    text
                } else {
                    format!("{text}{suffix}")
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        return Some(format!("dial:{rendered}"));
    }
    if let Some(rest) = fragment.strip_prefix("wait-friend-connected:")
        && let Some((name, deadline)) = rest.rsplit_once('@')
    {
        let id = orchestrator.wait_peer_id(name, token, live)?;
        return Some(format!("wait-friend-connected:{id}@{deadline}"));
    }
    if let Some(rest) = fragment.strip_prefix("wait-friend-disconnected:")
        && let Some((name, deadline)) = rest.rsplit_once('@')
    {
        let id = orchestrator.wait_peer_id(name, token, live)?;
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
    fn dial_renders_each_suffix_once_and_parses() {
        let id = libp2p::PeerId::random().to_string();
        let live = vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from(id.clone()),
                addrs: vec![
                    identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001"),
                    identity::PeerAddr::from("/ip4/127.0.0.1/udp/5002/quic-v1"),
                ],
            },
        }];
        let rendered =
            substitute_fragment(&orchestrator(), "dial:bob", "token", &live).expect("known peer");
        let suffix = format!("/p2p/{id}");
        let addrs = rendered
            .strip_prefix("dial:")
            .expect("dial prefix")
            .split_whitespace()
            .collect::<Vec<&str>>();
        assert_eq!(addrs.len(), 2);
        for addr in &addrs {
            assert!(addr.ends_with(suffix.as_str()), "missing suffix: {addr}");
            assert_eq!(addr.matches("/p2p/").count(), 1, "doubled suffix: {addr}");
            identity::Peer::try_from(*addr).expect("parses");
        }
    }

    #[test]
    fn suffixed_board_addrs_do_not_double_the_suffix() {
        let id = libp2p::PeerId::random().to_string();
        let bare = vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from(id.clone()),
                addrs: vec![
                    identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001"),
                    identity::PeerAddr::from("/ip4/127.0.0.1/udp/5002/quic-v1"),
                ],
            },
        }];
        let suffixed = vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from(id.clone()),
                addrs: vec![
                    identity::PeerAddr::from(format!("/ip4/127.0.0.1/tcp/5001/p2p/{id}")),
                    identity::PeerAddr::from(format!("/ip4/127.0.0.1/udp/5002/quic-v1/p2p/{id}")),
                ],
            },
        }];
        let from_bare =
            substitute_fragment(&orchestrator(), "dial:bob", "token", &bare).expect("bare");
        let from_suffixed =
            substitute_fragment(&orchestrator(), "dial:bob", "token", &suffixed).expect("suffixed");
        assert_eq!(from_bare, from_suffixed);
        assert!(!from_suffixed.contains(&format!("/p2p/{id}/p2p/")));
        let from_bare_friend =
            substitute_fragment(&orchestrator(), "add-friend:bob", "token", &bare)
                .expect("bare friend");
        let from_suffixed_friend =
            substitute_fragment(&orchestrator(), "add-friend:bob", "token", &suffixed)
                .expect("suffixed friend");
        assert_eq!(from_bare_friend, from_suffixed_friend);
        for fragment in from_suffixed_friend.split(", ") {
            let addr = fragment.strip_prefix("add-friend:").expect("friend prefix");
            assert_eq!(addr.matches("/p2p/").count(), 1, "{addr}");
            identity::Peer::try_from(addr).expect("parses");
        }
    }

    #[test]
    fn unknown_remote_waits_defer_the_peer() {
        let mut orchestrator = orchestrator();
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        assert_eq!(
            substitute_fragment(
                &orchestrator,
                "wait-friend-connected:bob@420s",
                "token",
                &[]
            ),
            None
        );
        assert_eq!(
            substitute_fragment(
                &orchestrator,
                "wait-friend-disconnected:bob@60s",
                "token",
                &[]
            ),
            None
        );
    }

    #[test]
    fn waits_prefer_live_ids_and_fall_back_to_stable_ids() {
        let rendered = substitute_fragment(
            &orchestrator(),
            "wait-friend-connected:bob@420s",
            "token",
            &live(),
        )
        .expect("live id");
        assert_eq!(rendered, "wait-friend-connected:bob-id@420s");
        let id = orchestrator()
            .peer_id_for_name("token", "bob")
            .expect("identity");
        let rendered = substitute_fragment(
            &orchestrator(),
            "wait-friend-disconnected:bob@60s",
            "token",
            &[],
        )
        .expect("stable id");
        assert_eq!(rendered, format!("wait-friend-disconnected:{id}@60s"));
        std::fs::remove_file(orchestrator().peer_identity_file("token", "bob")).expect("cleanup");
    }
}
