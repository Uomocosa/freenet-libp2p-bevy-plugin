use std::collections::BTreeMap;

use freenet_libp2p_bevy_plugin::{discovery, identity};

// needed helper: one lobby with a named and an unnamed member, as the plugin writes it
fn plugin_lobby() -> discovery::Lobby {
    let mut members = BTreeMap::new();
    for (peer, name) in [("alice-peer", Some("alice")), ("bob-peer", None)] {
        members.insert(
            identity::PeerId::from(peer),
            discovery::Heartbeat {
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/1")],
                updated_at: discovery::UnixTime::from_secs(7),
                display_name: name.map(identity::Username::from),
            },
        );
    }
    discovery::Lobby::from([(
        identity::RoomName::from("room-a"),
        discovery::Room {
            room_size: 8,
            members,
        },
    )])
}

#[test]
fn test_usage() {
    let plugin = plugin_lobby();
    let bytes = bincode::serialize(&plugin).unwrap_or_default();
    let contract: lobby_contract::LobbyState = bincode::deserialize(&bytes).unwrap_or_default();
    let room = contract.get(&lobby_contract::RoomName(String::from("room-a")));
    let alice = room.and_then(|room| {
        room.members
            .get(&lobby_contract::RemotePeerId(String::from("alice-peer")))
    });
    assert_eq!(
        alice.and_then(|heartbeat| heartbeat.display_name.as_deref()),
        Some("alice")
    );
    let back = bincode::serialize(&contract).unwrap_or_default();
    let round_trip: discovery::Lobby = bincode::deserialize(&back).unwrap_or_default();
    assert_eq!(round_trip, plugin);
}
