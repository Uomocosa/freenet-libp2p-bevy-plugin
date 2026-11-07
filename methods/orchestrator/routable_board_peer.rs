use crate::identity;
use crate::testing;

#[must_use]
pub fn routable_board_peer(peer: &testing::LivePeer) -> identity::Peer {
    let Some(lan) = testing::Orchestrator::discover_lan_ip() else {
        return peer.peer.clone();
    };
    let addrs = peer
        .peer
        .addrs
        .iter()
        .map(|addr| {
            let routed = addr
                .replace("/ip4/0.0.0.0/", &format!("/ip4/{lan}/"))
                .replace("/ip4/127.0.0.1/", &format!("/ip4/{lan}/"));
            identity::PeerAddr::from(routed)
        })
        .collect();
    identity::Peer {
        id: peer.peer.id.clone(),
        addrs,
    }
}

#[cfg(test)]
mod tests {
    use super::routable_board_peer;
    use crate::identity;
    use crate::testing;

    fn live(addrs: Vec<&str>) -> testing::LivePeer {
        testing::LivePeer {
            name: "alice".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from("alice-id"),
                addrs: addrs.into_iter().map(identity::PeerAddr::from).collect(),
            },
        }
    }

    #[test]
    fn test_usage() {
        let routed = routable_board_peer(&live(vec!["/ip4/0.0.0.0/tcp/5001"]));
        assert_eq!(routed.id, identity::PeerId::from("alice-id"));
        assert_eq!(routed.addrs.len(), 1);
        let rendered = routed.addrs[0].to_string();
        assert!(rendered.ends_with("/tcp/5001"));
        assert!(!rendered.contains("0.0.0.0"));
    }

    #[test]
    fn loopback_is_replaced_when_a_lan_exists() {
        if testing::Orchestrator::discover_lan_ip().is_none() {
            return;
        }
        let routed = routable_board_peer(&live(vec!["/ip4/127.0.0.1/tcp/5001"]));
        let rendered = routed.addrs[0].to_string();
        assert!(!rendered.contains("127.0.0.1"));
    }
}
