#![allow(unexpected_cfgs)]

use derive_more::Deref;
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::enums::RoomMembership;
pub use basic::newtypes::{PeerId, RoomName};
pub use basic::structs::{Peer, Room, RoomMember};
pub use basic::type_aliases::Lobby;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct UnixTime(pub u64);

#[rustfmt::skip]
impl From<u64> for UnixTime {
    fn from(secs: u64) -> Self { Self(secs) }
}

const MAX_ROOMS: usize = 256;
const MAX_MEMBERS_PER_ROOM: usize = 64;
const MAX_ROOM_NAME: usize = 64;

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
            if !Self::valid_room_name(room.as_str()) || record.room_size == 0 {
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
            let Some(bytes) = Self::decode_update(update) else {
                continue;
            };
            if bytes.is_empty() {
                continue;
            }
            let incoming = Self::decode_lobby(&bytes)?;
            merged = Self::merge_lobby(merged, incoming);
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

impl LobbyContract {
    pub(crate) fn valid_room_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= MAX_ROOM_NAME
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    pub(crate) fn merge_room_member(
        current: Option<RoomMember>,
        incoming: RoomMember,
    ) -> RoomMember {
        match current {
            Some(existing) if existing.updated_at >= incoming.updated_at => existing,
            _ => incoming,
        }
    }

    pub(crate) fn merge_room(current: Option<Room>, incoming: Room) -> Room {
        match current {
            Some(mut existing) if existing.created_at == incoming.created_at => {
                for (peer, room_member) in incoming.members {
                    let merged =
                        Self::merge_room_member(existing.members.remove(&peer), room_member);
                    existing.members.insert(peer, merged);
                }
                existing.room_size = existing.room_size.max(incoming.room_size);
                existing
            }
            Some(existing) if existing.created_at > incoming.created_at => existing,
            _ => incoming,
        }
    }

    pub(crate) fn merge_lobby(mut base: Lobby, incoming: Lobby) -> Lobby {
        for (room, record) in incoming {
            if !Self::valid_room_name(room.as_str()) {
                continue;
            }
            let merged = Self::merge_room(base.remove(&room), record);
            if merged.room_size == 0 || merged.members.len() > MAX_MEMBERS_PER_ROOM {
                continue;
            }
            base.insert(room, merged);
        }
        Self::drop_rooms_where_every_member_quit(&mut base);
        Self::evict_oldest_rooms_until_lobby_fits_capacity(&mut base, MAX_ROOMS);
        base
    }

    pub(crate) fn drop_rooms_where_every_member_quit(lobby: &mut Lobby) {
        lobby.retain(|_, room| {
            room.members
                .values()
                .any(|room_member| room_member.membership != RoomMembership::Departed)
        });
    }

    pub(crate) fn newest_room_member(room: &Room) -> UnixTime {
        room.members
            .values()
            .map(|room_member| room_member.updated_at)
            .max()
            .unwrap_or_else(|| UnixTime::from(0))
    }

    pub(crate) fn evict_oldest_rooms_until_lobby_fits_capacity(lobby: &mut Lobby, capacity: usize) {
        while lobby.len() > capacity {
            let oldest = lobby
                .iter()
                .min_by_key(|(name, room)| (Self::newest_room_member(room), name.as_str()))
                .map(|(name, _)| name.clone());
            let Some(oldest) = oldest else {
                break;
            };
            lobby.remove(&oldest);
        }
    }

    pub(crate) fn decode_update(data: UpdateData<'static>) -> Option<Vec<u8>> {
        match data {
            UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
                Some(state.as_ref().to_vec())
            }
            UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
            _ => None,
        }
    }

    pub(crate) fn decode_lobby(bytes: &[u8]) -> Result<Lobby, ContractError> {
        bincode::deserialize(bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
            reason: e.to_string(),
        })
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
            RoomMember {
                peer: Peer {
                    id: PeerId(peer.to_string()),
                    addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
                },
                updated_at,
                display_name: None,
                membership: RoomMembership::Occupant,
            },
        );
        Room {
            room_size: 8,
            created_at,
            members,
        }
    }

    fn room_at(updated_at: UnixTime) -> Room {
        let mut members = BTreeMap::new();
        members.insert(
            PeerId("peer".to_string()),
            RoomMember {
                peer: Peer {
                    id: PeerId("peer".to_string()),
                    addrs: Vec::new(),
                },
                updated_at,
                display_name: None,
                membership: RoomMembership::Occupant,
            },
        );
        Room {
            room_size: 8,
            created_at: UnixTime::from(1),
            members,
        }
    }

    fn departed_room(updated_at: UnixTime) -> Room {
        let mut record = room_at(updated_at);
        for heartbeat in record.members.values_mut() {
            heartbeat.membership = RoomMembership::Departed;
        }
        record
    }

    fn lobby_with_rooms(count: usize, newest_at: impl Fn(usize) -> UnixTime) -> Lobby {
        (0..count)
            .map(|index| {
                (
                    RoomName(format!("room-{index:03}")),
                    room_at(newest_at(index)),
                )
            })
            .collect()
    }

    fn apply(params: &Parameters<'static>, base: &Lobby, incoming: &Lobby) -> Lobby {
        let update = UpdateData::Delta(StateDelta::from(bincode::serialize(incoming).unwrap()));
        let merged =
            LobbyContract::update_state(params.clone(), state_of(base), vec![update]).unwrap();
        bincode::deserialize(&state_bytes(merged)).unwrap()
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
            room("peer", UnixTime::from(5), UnixTime::from(10)),
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
                    room("peer", UnixTime::from(1), UnixTime::from(10)),
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
            UnixTime::from(5)
        );
    }

    #[test]
    fn recreated_room_replaces_old_generation_both_orders() {
        let params = Parameters::from(Vec::new());
        let room_name = RoomName("room-a".to_string());
        let low = room("peer", UnixTime::from(1), UnixTime::from(100));
        let high = room("peer", UnixTime::from(1), UnixTime::from(500));

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
                UnixTime::from(500),
                "the newer generation must win in either merge order"
            );
        }
    }

    #[test]
    fn departed_tombstone_wins_over_older_heartbeat() {
        let params = Parameters::from(Vec::new());
        let name = RoomName("room-a".to_string());
        let live = room("peer", UnixTime::from(5), UnixTime::from(1));
        let mut departed = room("peer", UnixTime::from(9), UnixTime::from(1));
        for heartbeat in departed.members.values_mut() {
            heartbeat.membership = RoomMembership::Departed;
        }
        let base = Lobby::from([(name.clone(), live)]);
        let incoming = Lobby::from([(name.clone(), departed)]);
        let update = UpdateData::Delta(StateDelta::from(bincode::serialize(&incoming).unwrap()));
        let merged = LobbyContract::update_state(params, state_of(&base), vec![update]).unwrap();
        let decoded: Lobby = bincode::deserialize(&state_bytes(merged)).unwrap();
        assert!(
            !decoded.contains_key(&name),
            "the departed tombstone must win over the older live heartbeat"
        );
    }

    #[test]
    fn all_departed_room_is_dropped_in_either_merge_order() {
        let params = Parameters::from(Vec::new());
        let name = RoomName("room-a".to_string());
        let live = room_at(UnixTime::from(5));
        let departed = departed_room(UnixTime::from(9));
        let pairs = [(live.clone(), departed.clone()), (departed, live)];
        for (base_room, incoming_room) in pairs {
            let base = Lobby::from([(name.clone(), base_room)]);
            let incoming = Lobby::from([(name.clone(), incoming_room)]);
            let merged = apply(&params, &base, &incoming);
            assert!(
                !merged.contains_key(&name),
                "an all-departed room must not survive the merge"
            );
        }
    }

    #[test]
    fn stale_live_heartbeat_resurrects_a_dropped_room() {
        let params = Parameters::from(Vec::new());
        let name = RoomName("room-a".to_string());
        let dead = Lobby::from([(name.clone(), departed_room(UnixTime::from(9)))]);
        let dropped = apply(&params, &Lobby::new(), &dead);
        assert!(!dropped.contains_key(&name));
        let stale = Lobby::from([(name.clone(), room_at(UnixTime::from(5)))]);
        let revived = apply(&params, &dropped, &stale);
        assert!(revived.contains_key(&name));
    }

    #[test]
    fn all_departed_room_never_displaces_a_live_room() {
        let params = Parameters::from(Vec::new());
        let base = lobby_with_rooms(MAX_ROOMS, |index| {
            UnixTime::from(u64::try_from(index).unwrap_or_default().saturating_add(1))
        });
        let dead = Lobby::from([(
            RoomName("z-dead".to_string()),
            departed_room(UnixTime::from(1_000)),
        )]);
        let merged = apply(&params, &base, &dead);
        assert_eq!(merged.len(), MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName("z-dead".to_string())));
        assert!(merged.contains_key(&RoomName("room-000".to_string())));
    }

    #[test]
    fn cap_evicts_the_oldest_room_and_keeps_the_incoming_one() {
        let params = Parameters::from(Vec::new());
        let base = lobby_with_rooms(MAX_ROOMS, |index| {
            UnixTime::from(u64::try_from(index).unwrap_or_default().saturating_add(1))
        });
        let incoming = Lobby::from([(
            RoomName("z-new".to_string()),
            room_at(UnixTime::from(1_000)),
        )]);
        let merged = apply(&params, &base, &incoming);
        assert_eq!(merged.len(), MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName("room-000".to_string())));
        assert!(merged.contains_key(&RoomName("z-new".to_string())));
    }

    #[test]
    fn cap_evicts_the_smaller_name_when_newest_heartbeats_tie() {
        let params = Parameters::from(Vec::new());
        let base = lobby_with_rooms(MAX_ROOMS, |_| UnixTime::from(5));
        let incoming = Lobby::from([(RoomName("z-new".to_string()), room_at(UnixTime::from(5)))]);
        let merged = apply(&params, &base, &incoming);
        assert_eq!(merged.len(), MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName("room-000".to_string())));
        assert!(merged.contains_key(&RoomName("z-new".to_string())));
    }

    #[test]
    fn cap_evicts_the_incoming_room_when_it_is_the_oldest() {
        let params = Parameters::from(Vec::new());
        let base = lobby_with_rooms(MAX_ROOMS, |_| UnixTime::from(100));
        let incoming =
            Lobby::from([(RoomName("new-room".to_string()), room_at(UnixTime::from(1)))]);
        let merged = apply(&params, &base, &incoming);
        assert_eq!(merged.len(), MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName("new-room".to_string())));
        assert!(merged.contains_key(&RoomName("room-000".to_string())));
    }

    #[test]
    fn evicts_the_oldest_rooms_until_the_lobby_fits() {
        let mut lobby = lobby_with_rooms(3, |index| {
            UnixTime::from(u64::try_from(index).unwrap_or_default())
        });
        LobbyContract::evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&RoomName("room-000".to_string())));
        assert!(lobby.contains_key(&RoomName("room-002".to_string())));
    }
}
