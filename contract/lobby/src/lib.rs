#![allow(unexpected_cfgs)]

use derive_more::Deref;
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::newtypes::{PeerId, RoomName};
pub use basic::structs::{Heartbeat, Room};
pub use basic::type_aliases::Lobby;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct UnixTime(pub u64);

impl UnixTime {
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs)
    }
}

const MAX_ROOMS: usize = 256;
const MAX_MEMBERS_PER_ROOM: usize = 64;
const MAX_ROOM_NAME: usize = 64;

fn valid_room_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_ROOM_NAME
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn merge_heartbeat(current: Option<Heartbeat>, incoming: Heartbeat) -> Heartbeat {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

fn merge_room(current: Option<Room>, incoming: Room) -> Room {
    match current {
        Some(mut existing) => {
            for (peer, heartbeat) in incoming.members {
                let merged = merge_heartbeat(existing.members.remove(&peer), heartbeat);
                existing.members.insert(peer, merged);
            }
            existing.room_size = existing.room_size.max(incoming.room_size);
            existing.created_at = existing.created_at.min(incoming.created_at);
            existing
        }
        None => incoming,
    }
}

fn merge_lobby(mut base: Lobby, incoming: Lobby) -> Lobby {
    for (room, record) in incoming {
        if !valid_room_name(room.as_str()) {
            continue;
        }
        let merged = merge_room(base.remove(&room), record);
        if merged.room_size == 0 || merged.members.len() > MAX_MEMBERS_PER_ROOM {
            continue;
        }
        if !base.contains_key(&room) && base.len() >= MAX_ROOMS {
            continue;
        }
        base.insert(room, merged);
    }
    base
}

fn decode_update(data: UpdateData<'static>) -> Option<Vec<u8>> {
    match data {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
            Some(state.as_ref().to_vec())
        }
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        _ => None,
    }
}

fn decode_lobby(bytes: &[u8]) -> Result<Lobby, ContractError> {
    bincode::deserialize(bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
        reason: e.to_string(),
    })
}

pub struct LobbyContract;

#[contract]
impl ContractInterface for LobbyContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let lobby: Lobby =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        if lobby.len() > MAX_ROOMS {
            return Err(ContractError::InvalidState);
        }
        for (room, record) in &lobby {
            if !valid_room_name(room.as_str()) || record.room_size == 0 {
                return Err(ContractError::InvalidState);
            }
            if record.members.len() > MAX_MEMBERS_PER_ROOM {
                return Err(ContractError::InvalidState);
            }
        }
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut merged: Lobby = bincode::deserialize(state.as_ref()).unwrap_or_default();
        for update in data {
            let Some(bytes) = decode_update(update) else {
                continue;
            };
            if bytes.is_empty() {
                continue;
            }
            let incoming = decode_lobby(&bytes)?;
            merged = merge_lobby(merged, incoming);
        }
        Ok(UpdateModification::valid(State::from(bincode::serialize(
            &merged,
        )?)))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let lobby: Lobby =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        Ok(StateSummary::from(bincode::serialize(&lobby)?))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        let ours: Lobby =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        if summary.as_ref() == bincode::serialize(&ours)?.as_slice() {
            return Ok(StateDelta::from(Vec::new()));
        }
        Ok(StateDelta::from(state.as_ref().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn state_of(lobby: &Lobby) -> State<'static> {
        State::from(bincode::serialize(lobby).unwrap())
    }

    fn state_bytes(modification: UpdateModification<'_>) -> Vec<u8> {
        modification
            .new_state
            .map_or_else(Vec::new, |state| state.as_ref().to_vec())
    }

    fn room(peer: &str, updated_at: UnixTime, created_at: UnixTime) -> Room {
        let mut members = BTreeMap::new();
        members.insert(
            PeerId(peer.to_string()),
            Heartbeat {
                addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
                updated_at,
                display_name: None,
            },
        );
        Room {
            room_size: 8,
            created_at,
            members,
        }
    }

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let empty = state_of(&Lobby::default());
        assert!(matches!(
            LobbyContract::validate_state(params.clone(), empty, related.clone()),
            Ok(ValidateResult::Valid)
        ));

        let mut base = Lobby::new();
        base.insert(
            RoomName("room-a".to_string()),
            room("peer", UnixTime::from_secs(5), UnixTime::from_secs(10)),
        );
        let update = UpdateData::Delta(StateDelta::from(bincode::serialize(&base).unwrap()));
        let merged =
            LobbyContract::update_state(params.clone(), state_of(&Lobby::default()), vec![update])
                .unwrap();
        let bytes = state_bytes(merged);
        let decoded: Lobby = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.len(), 1);

        let older = UpdateData::Delta(StateDelta::from(
            bincode::serialize(&{
                let mut old = Lobby::new();
                old.insert(
                    RoomName("room-a".to_string()),
                    room("peer", UnixTime::from_secs(1), UnixTime::from_secs(10)),
                );
                old
            })
            .unwrap(),
        ));
        let merged = LobbyContract::update_state(params, State::from(bytes), vec![older]).unwrap();
        let bytes = state_bytes(merged);
        let decoded: Lobby = bincode::deserialize(&bytes).unwrap();
        assert_eq!(
            decoded[&RoomName("room-a".to_string())].members[&PeerId("peer".to_string())]
                .updated_at,
            UnixTime::from_secs(5)
        );
    }

    #[test]
    fn merge_keeps_min_created_at_both_orders() {
        let params = Parameters::from(Vec::new());
        let room_name = RoomName("room-a".to_string());
        let low = room("peer", UnixTime::from_secs(1), UnixTime::from_secs(100));
        let high = room("peer", UnixTime::from_secs(1), UnixTime::from_secs(500));

        for (first, second) in [(low.clone(), high.clone()), (high, low)] {
            let mut base = Lobby::new();
            base.insert(room_name.clone(), first);
            let incoming = {
                let mut other = Lobby::new();
                other.insert(room_name.clone(), second);
                other
            };
            let update =
                UpdateData::Delta(StateDelta::from(bincode::serialize(&incoming).unwrap()));
            let merged =
                LobbyContract::update_state(params.clone(), state_of(&base), vec![update]).unwrap();
            let decoded: Lobby = bincode::deserialize(&state_bytes(merged)).unwrap();
            assert_eq!(
                decoded[&room_name].created_at,
                UnixTime::from_secs(100),
                "min created_at must survive either merge order"
            );
        }
    }
}
