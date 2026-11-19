#![allow(unexpected_cfgs)]

use atomic_delegate_macros::{atomic_delegate, atomic_delegates};
use freenet_stdlib::prelude::*;

use crate::Lobby;
use crate::Room;
use crate::constants;

pub struct LobbyContract;

#[contract]
impl ContractInterface for LobbyContract {
    #[atomic_delegate(LobbyContract, ContractInterface)]
    fn validate_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut merged: Lobby = bincode::deserialize(state.as_ref()).unwrap_or_default();
        for update in data {
            let Some(bytes) = Self::decode_update(&update) else {
                continue;
            };
            if bytes.is_empty() {
                continue;
            }
            let incoming = Self::decode_lobby(&bytes)?;
            let Lobby(rooms) = incoming;
            for (room, record) in rooms {
                if !Self::valid_room_name(room.as_str()) {
                    continue;
                }
                let combined = Room::merge_room(merged.remove(&room), record);
                if combined.members.len() > constants::MAX_MEMBERS_PER_ROOM {
                    continue;
                }
                merged.insert(room, combined);
            }
            merged.drop_rooms_where_every_member_quit();
            merged.evict_oldest_rooms_until_lobby_fits_capacity(constants::MAX_ROOMS);
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

#[atomic_delegates]
impl LobbyContract {
    pub fn decode_lobby(bytes: &[u8]) -> Result<Lobby, ContractError> {}
    pub fn decode_update(data: &UpdateData<'static>) -> Option<Vec<u8>> {}
    pub fn valid_room_name(name: &str) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lobby;
    use crate::PeerId;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::UnixTime;
    use crate::testing;

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let empty = testing::state_of(&Lobby::default());
        assert!(matches!(
            LobbyContract::validate_state(params.clone(), empty, related.clone()),
            Ok(ValidateResult::Valid)
        ));

        let mut base = Lobby::default();
        base.insert(
            RoomName::from("room-a"),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(10)),
        );
        let update = UpdateData::Delta(StateDelta::from(bincode::serialize(&base).unwrap()));
        let merged = LobbyContract::update_state(
            params.clone(),
            testing::state_of(&Lobby::default()),
            vec![update],
        )
        .unwrap();
        let bytes = testing::state_bytes(merged);
        let decoded: Lobby = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.len(), 1);

        let older = UpdateData::Delta(StateDelta::from(
            bincode::serialize(&{
                let mut old = Lobby::default();
                old.insert(
                    RoomName::from("room-a"),
                    testing::room("peer", UnixTime::from_secs(1), UnixTime::from_secs(10)),
                );
                old
            })
            .unwrap(),
        ));
        let merged = LobbyContract::update_state(params, State::from(bytes), vec![older]).unwrap();
        let bytes = testing::state_bytes(merged);
        let decoded: Lobby = bincode::deserialize(&bytes).unwrap();
        assert_eq!(
            decoded[&RoomName::from("room-a")].members[&PeerId("peer".to_string())].updated_at,
            UnixTime::from_secs(5)
        );
    }

    #[test]
    fn recreated_room_replaces_old_generation_both_orders() {
        let params = Parameters::from(Vec::new());
        let room_name = RoomName::from("room-a");
        let low = testing::room("peer", UnixTime::from_secs(1), UnixTime::from_secs(100));
        let high = testing::room("peer", UnixTime::from_secs(1), UnixTime::from_secs(500));

        for (first, second) in [(low.clone(), high.clone()), (high, low)] {
            let mut base = Lobby::default();
            base.insert(room_name.clone(), first);
            let incoming = {
                let mut other = Lobby::default();
                other.insert(room_name.clone(), second);
                other
            };
            let update =
                UpdateData::Delta(StateDelta::from(bincode::serialize(&incoming).unwrap()));
            let merged =
                LobbyContract::update_state(params.clone(), testing::state_of(&base), vec![update])
                    .unwrap();
            let decoded: Lobby = bincode::deserialize(&testing::state_bytes(merged)).unwrap();
            assert_eq!(
                decoded[&room_name].created_at,
                UnixTime::from_secs(500),
                "the newer generation must win in either merge order"
            );
        }
    }

    #[test]
    fn departed_tombstone_wins_over_older_heartbeat() {
        let params = Parameters::from(Vec::new());
        let name = RoomName::from("room-a");
        let live = testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(1));
        let mut departed = testing::room("peer", UnixTime::from_secs(9), UnixTime::from_secs(1));
        for heartbeat in departed.members.values_mut() {
            heartbeat.membership = RoomMembership::Departed;
        }
        let base = Lobby::from([(name.clone(), live)]);
        let incoming = Lobby::from([(name.clone(), departed)]);
        let update = UpdateData::Delta(StateDelta::from(bincode::serialize(&incoming).unwrap()));
        let merged =
            LobbyContract::update_state(params, testing::state_of(&base), vec![update]).unwrap();
        let decoded: Lobby = bincode::deserialize(&testing::state_bytes(merged)).unwrap();
        assert!(
            !decoded.contains_key(&name),
            "the departed tombstone must win over the older live heartbeat"
        );
    }

    #[test]
    fn all_departed_room_is_dropped_in_either_merge_order() {
        let params = Parameters::from(Vec::new());
        let name = RoomName::from("room-a");
        let live = testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(1));
        let departed = testing::departed_room(UnixTime::from_secs(9));
        let pairs = [(live.clone(), departed.clone()), (departed, live)];
        for (base_room, incoming_room) in pairs {
            let base = Lobby::from([(name.clone(), base_room)]);
            let incoming = Lobby::from([(name.clone(), incoming_room)]);
            let merged = testing::apply(&params, &base, &incoming);
            assert!(
                !merged.contains_key(&name),
                "an all-departed room must not survive the merge"
            );
        }
    }

    #[test]
    fn stale_live_heartbeat_resurrects_a_dropped_room() {
        let params = Parameters::from(Vec::new());
        let name = RoomName::from("room-a");
        let dead = Lobby::from([(name.clone(), testing::departed_room(UnixTime::from_secs(9)))]);
        let dropped = testing::apply(&params, &Lobby::default(), &dead);
        assert!(!dropped.contains_key(&name));
        let stale = Lobby::from([(
            name.clone(),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(1)),
        )]);
        let revived = testing::apply(&params, &dropped, &stale);
        assert!(revived.contains_key(&name));
    }

    #[test]
    fn all_departed_room_never_displaces_a_live_room() {
        let params = Parameters::from(Vec::new());
        let base = testing::lobby_with_rooms(constants::MAX_ROOMS, |index| {
            UnixTime::from_secs(u64::try_from(index).unwrap_or_default().saturating_add(1))
        });
        let dead = Lobby::from([(
            RoomName::from("z-dead"),
            testing::departed_room(UnixTime::from_secs(1_000)),
        )]);
        let merged = testing::apply(&params, &base, &dead);
        assert_eq!(merged.len(), constants::MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName::from("z-dead")));
        assert!(merged.contains_key(&RoomName::from("room-000")));
    }

    #[test]
    fn cap_evicts_the_oldest_room_and_keeps_the_incoming_one() {
        let params = Parameters::from(Vec::new());
        let base = testing::lobby_with_rooms(constants::MAX_ROOMS, |index| {
            UnixTime::from_secs(u64::try_from(index).unwrap_or_default().saturating_add(1))
        });
        let incoming = Lobby::from([(
            RoomName::from("z-new"),
            testing::room("peer", UnixTime::from_secs(1_000), UnixTime::from_secs(1)),
        )]);
        let merged = testing::apply(&params, &base, &incoming);
        assert_eq!(merged.len(), constants::MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName::from("room-000")));
        assert!(merged.contains_key(&RoomName::from("z-new")));
    }

    #[test]
    fn cap_evicts_the_smaller_name_when_newest_heartbeats_tie() {
        let params = Parameters::from(Vec::new());
        let base = testing::lobby_with_rooms(constants::MAX_ROOMS, |_| UnixTime::from_secs(5));
        let incoming = Lobby::from([(
            RoomName::from("z-new"),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(1)),
        )]);
        let merged = testing::apply(&params, &base, &incoming);
        assert_eq!(merged.len(), constants::MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName::from("room-000")));
        assert!(merged.contains_key(&RoomName::from("z-new")));
    }

    #[test]
    fn cap_evicts_the_incoming_room_when_it_is_the_oldest() {
        let params = Parameters::from(Vec::new());
        let base = testing::lobby_with_rooms(constants::MAX_ROOMS, |_| UnixTime::from_secs(100));
        let incoming = Lobby::from([(
            RoomName::from("new-room"),
            testing::room("peer", UnixTime::from_secs(1), UnixTime::from_secs(1)),
        )]);
        let merged = testing::apply(&params, &base, &incoming);
        assert_eq!(merged.len(), constants::MAX_ROOMS);
        assert!(!merged.contains_key(&RoomName::from("new-room")));
        assert!(merged.contains_key(&RoomName::from("room-000")));
    }

    #[test]
    fn evicts_the_oldest_rooms_until_the_lobby_fits() {
        let mut lobby = testing::lobby_with_rooms(3, |index| {
            UnixTime::from_secs(u64::try_from(index).unwrap_or_default())
        });
        lobby.evict_oldest_rooms_until_lobby_fits_capacity(2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&RoomName::from("room-000")));
        assert!(lobby.contains_key(&RoomName::from("room-002")));
    }
}
