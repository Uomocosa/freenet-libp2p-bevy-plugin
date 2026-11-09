use std::collections::BTreeMap;

use freenet_contract_harness::{SuiteConfig, run_suite};
use freenet_stdlib::prelude::*;
use lobby_contract::{
    Lobby, LobbyContract, LobbyRoom, Peer, PeerAddr, PeerId, RoomMember, RoomName, RoomSize,
    UnixTime,
};

fn encode(lobby: &Lobby) -> State<'static> {
    State::from(bincode::serialize(lobby).unwrap_or_default())
}

// needed helper: a lobby with one room holding one member at the given times
fn one_room(updated_at: u64, created_at: u64) -> Lobby {
    let mut members = BTreeMap::new();
    members.insert(
        PeerId::from("peer"),
        RoomMember {
            peer: Peer {
                id: PeerId::from("peer"),
                addrs: vec![PeerAddr::from("/ip4/127.0.0.1/tcp/1")],
            },
            updated_at: UnixTime::from_secs(updated_at),
            display_name: None,
            membership: lobby_contract::RoomMembership::Occupant,
        },
    );
    let mut lobby = Lobby::default();
    lobby.insert(
        RoomName::from("room-a"),
        LobbyRoom {
            room_size: RoomSize::DEFAULT,
            created_at: UnixTime::from_secs(created_at),
            members,
        },
    );
    lobby
}

fn gen_state() -> State<'static> {
    encode(&one_room(1, 100))
}

fn gen_update(_base: &State<'static>) -> UpdateData<'static> {
    let delta = bincode::serialize(&one_room(2, 200)).unwrap_or_default();
    UpdateData::Delta(StateDelta::from(delta))
}

fn empty_state() -> State<'static> {
    encode(&Lobby::default())
}

#[test]
#[ignore = "freenet harness: contract CRDT suite"]
fn test_usage() {
    run_suite::<LobbyContract>(&SuiteConfig {
        params: Parameters::from(Vec::new()),
        gen_state,
        gen_update,
        gen_divergent_equal_total: None,
        empty_state,
    });
}
