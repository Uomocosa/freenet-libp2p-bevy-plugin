use std::collections::BTreeMap;

use freenet_contract_harness::{SuiteConfig, run_suite};
use freenet_stdlib::prelude::*;
use lobby_contract::{
    LobbyContract, LobbyState, Presence, RemotePeerId, RoomName, RoomRecord, UnixTime,
};

fn encode(lobby: &LobbyState) -> State<'static> {
    State::from(bincode::serialize(lobby).unwrap_or_default())
}

// needed helper: a lobby with one room holding one member at the given time
fn one_room(updated_at: u64) -> LobbyState {
    let mut members = BTreeMap::new();
    members.insert(
        RemotePeerId("peer".to_string()),
        Presence {
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
            updated_at: UnixTime(updated_at),
        },
    );
    let mut lobby = LobbyState::new();
    lobby.insert(
        RoomName("room-a".to_string()),
        RoomRecord {
            capacity: 8,
            members,
        },
    );
    lobby
}

fn gen_state() -> State<'static> {
    encode(&one_room(1))
}

fn gen_update(_base: &State<'static>) -> UpdateData<'static> {
    let delta = bincode::serialize(&one_room(2)).unwrap_or_default();
    UpdateData::Delta(StateDelta::from(delta))
}

fn empty_state() -> State<'static> {
    encode(&LobbyState::new())
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
