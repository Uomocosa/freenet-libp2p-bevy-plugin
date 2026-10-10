#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoomName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RemotePeerId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixTime(pub u64);

impl UnixTime {
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub addrs: Vec<String>,
    pub updated_at: UnixTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomRecord {
    pub capacity: u16,
    pub members: BTreeMap<RemotePeerId, Presence>,
}

pub type LobbyState = BTreeMap<RoomName, RoomRecord>;

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

fn merge_presence(current: Option<Presence>, incoming: Presence) -> Presence {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

fn merge_room(current: Option<RoomRecord>, incoming: RoomRecord) -> RoomRecord {
    match current {
        Some(mut existing) => {
            for (peer, presence) in incoming.members {
                let merged = merge_presence(existing.members.remove(&peer), presence);
                existing.members.insert(peer, merged);
            }
            existing.capacity = existing.capacity.max(incoming.capacity);
            existing
        }
        None => incoming,
    }
}

fn merge_lobby(mut base: LobbyState, incoming: LobbyState) -> LobbyState {
    for (room, record) in incoming {
        if !valid_room_name(&room.0) {
            continue;
        }
        let merged = merge_room(base.remove(&room), record);
        if merged.capacity == 0 || merged.members.len() > MAX_MEMBERS_PER_ROOM {
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
        UpdateData::State(state) => Some(state.as_ref().to_vec()),
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        UpdateData::StateAndDelta { state, .. } => Some(state.as_ref().to_vec()),
        _ => None,
    }
}

fn decode_lobby(bytes: &[u8]) -> Result<LobbyState, ContractError> {
    bincode::deserialize(bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
        reason: e.to_string(),
    })
}

#[allow(dead_code)]
struct LobbyContract;

#[contract]
impl ContractInterface for LobbyContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let lobby: LobbyState =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        if lobby.len() > MAX_ROOMS {
            return Err(ContractError::InvalidState);
        }
        for (room, record) in &lobby {
            if !valid_room_name(&room.0) || record.capacity == 0 {
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
        let mut merged: LobbyState =
            bincode::deserialize(state.as_ref()).unwrap_or_default();
        for update in data {
            let Some(bytes) = decode_update(update) else {
                continue;
            };
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
        let lobby: LobbyState =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        Ok(StateSummary::from(bincode::serialize(&lobby)?))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        Ok(StateDelta::from(state.as_ref().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_of(lobby: LobbyState) -> State<'static> {
        State::from(bincode::serialize(&lobby).unwrap())
    }

    fn state_bytes(modification: UpdateModification<'_>) -> Vec<u8> {
        modification
            .new_state
            .map_or_else(Vec::new, |state| state.as_ref().to_vec())
    }

    fn room(peer: &str, updated_at: UnixTime) -> RoomRecord {
        let mut members = BTreeMap::new();
        members.insert(
            RemotePeerId(peer.to_string()),
            Presence {
                addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
                updated_at,
            },
        );
        RoomRecord {
            capacity: 8,
            members,
        }
    }

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let empty = state_of(LobbyState::default());
        assert!(matches!(
            LobbyContract::validate_state(params.clone(), empty, related.clone()),
            Ok(ValidateResult::Valid)
        ));

        let mut base = LobbyState::new();
        base.insert(RoomName("room-a".to_string()), room("peer", UnixTime::from_secs(5)));
        let update = UpdateData::Delta(StateDelta::from(
            bincode::serialize(&base).unwrap(),
        ));
        let merged = LobbyContract::update_state(
            params.clone(),
            state_of(LobbyState::default()),
            vec![update],
        )
        .unwrap();
        let bytes = state_bytes(merged);
        let decoded: LobbyState = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.len(), 1);

        let older = UpdateData::Delta(StateDelta::from(
            bincode::serialize(&{
                let mut old = LobbyState::new();
                old.insert(RoomName("room-a".to_string()), room("peer", UnixTime::from_secs(1)));
                old
            })
            .unwrap(),
        ));
        let merged = LobbyContract::update_state(
            params,
            State::from(bytes),
            vec![older],
        )
        .unwrap();
        let bytes = state_bytes(merged);
        let decoded: LobbyState = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded[&RoomName("room-a".to_string())].members[&RemotePeerId("peer".to_string())].updated_at, UnixTime::from_secs(5));
    }
}
