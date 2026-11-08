use std::collections::{BTreeMap, BTreeSet};

use atomic_delegate_macros::atomic_delegates;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::testing;
use crate::time;

pub struct PeerMessageRouter {
    pub(crate) game: identity::GameId,
    pub(crate) registered_peers: BTreeSet<identity::PeerId>,
    pub(crate) connected_peers: BTreeMap<identity::PeerId, BTreeSet<identity::PeerId>>,
    pub(crate) subscribed_topics: BTreeMap<identity::PeerId, BTreeSet<identity::Topic>>,
    pub(crate) joined_rooms: BTreeMap<identity::PeerId, identity::RoomName>,
    pub(crate) heartbeat_targets:
        BTreeMap<identity::PeerId, Vec<discovery::state_machine::HeartbeatTarget>>,
    pub(crate) shared_lobby: discovery::Lobby,
    pub(crate) history_values: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) history_kinds: BTreeMap<Vec<u8>, p2p::RoomHistoryRecordKind>,
    pub(crate) pending_snapshot_replies: BTreeMap<p2p::ReplyId, identity::PeerId>,
    pub(crate) next_reply_id: u64,
}

impl PeerMessageRouter {
    #[must_use]
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            registered_peers: BTreeSet::new(),
            connected_peers: BTreeMap::new(),
            subscribed_topics: BTreeMap::new(),
            joined_rooms: BTreeMap::new(),
            heartbeat_targets: BTreeMap::new(),
            shared_lobby: discovery::Lobby::default(),
            history_values: BTreeMap::new(),
            history_kinds: BTreeMap::new(),
            pending_snapshot_replies: BTreeMap::new(),
            next_reply_id: 0,
        }
    }
}

#[atomic_delegates]
impl PeerMessageRouter {
    pub fn register_peer(&mut self, peer: &identity::Peer) {}
    pub fn remove_peer(&mut self, peer: &identity::PeerId) -> Vec<testing::RoutedEvent> {}
    pub fn route(
        &mut self,
        from: &identity::PeerId,
        command: p2p::Libp2pCommand,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub fn refresh_lobby(
        &mut self,
        peer: &identity::PeerId,
        targets: &[discovery::state_machine::HeartbeatTarget],
        now: time::UnixTime,
    ) -> discovery::Lobby {
    }
    pub fn route_joined_room_bytes(
        &mut self,
        from: &identity::PeerId,
        data: Vec<u8>,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub fn route_specific_room_bytes(
        &mut self,
        from: &identity::PeerId,
        room: &identity::RoomName,
        data: Vec<u8>,
    ) -> Vec<testing::RoutedEvent> {
    }
}

#[atomic_delegates]
impl PeerMessageRouter {
    pub(crate) fn claim_reply_id(&mut self, requester: &identity::PeerId) -> p2p::ReplyId {}
    pub(crate) fn connect_dial_target(
        &mut self,
        from: &identity::PeerId,
        peer: identity::Peer,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub(crate) fn update_peer_subscriptions(
        &mut self,
        from: &identity::PeerId,
        command: p2p::Libp2pCommand,
    ) -> Vec<testing::RoutedEvent> {
    }
}

#[rustfmt::skip]
impl PeerMessageRouter {
    pub(crate) fn is_connected(&self, first: &identity::PeerId, second: &identity::PeerId) -> bool {
        self.connected_peers.get(first).is_some_and(|peers| peers.contains(second))
    }
    pub(crate) fn serve_history_command(&mut self, from: &identity::PeerId, command: p2p::Libp2pCommand) -> Vec<testing::RoutedEvent> {
        match command {
            p2p::Libp2pCommand::PutHistoryRecord { id, value, .. } => {
                let key: Vec<u8> = id.history_record_key().as_ref().to_vec();
                self.history_values.insert(key.clone(), value);
                self.history_kinds.insert(key, id.clone());
                vec![testing::RoutedEvent {
                    to: from.clone(),
                    event: p2p::Libp2pEvent::HistoryRecordStored { id },
                }]
            }
            p2p::Libp2pCommand::GetHistoryRecord { id } => {
                let key: Vec<u8> = id.history_record_key().as_ref().to_vec();
                let event = self.history_values.get(&key).map_or_else(
                    || p2p::Libp2pEvent::HistoryRecordNotFound { id: id.clone() },
                    |value| p2p::Libp2pEvent::HistoryRecordFound {
                        id: id.clone(),
                        value: value.clone(),
                    },
                );
                vec![testing::RoutedEvent {
                    to: from.clone(),
                    event,
                }]
            }
            p2p::Libp2pCommand::MeasureHistoryStore => {
                let mut bytes_by_prefix: BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount> =
                    BTreeMap::new();
                for (key, value) in &self.history_values {
                    let Some(kind) = self.history_kinds.get(key) else {
                        continue;
                    };
                    let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
                    let total: &mut p2p::HistoryByteCount = bytes_by_prefix
                        .entry(kind.record_prefix().clone())
                        .or_default();
                    *total = p2p::HistoryByteCount::from(total.saturating_add(bytes));
                }
                vec![testing::RoutedEvent {
                    to: from.clone(),
                    event: p2p::Libp2pEvent::HistoryStoreMeasured { bytes_by_prefix },
                }]
            }
            p2p::Libp2pCommand::RemoveHistoryRecords { prefix } => {
                let stale: Vec<Vec<u8>> = self
                    .history_kinds
                    .iter()
                    .filter(|(_, kind)| *kind.record_prefix() == prefix)
                    .map(|(key, _)| key.clone())
                    .collect();
                for key in stale {
                    self.history_values.remove(&key);
                    self.history_kinds.remove(&key);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
    pub(crate) fn serve_room_sync_command(&mut self, from: &identity::PeerId, command: p2p::Libp2pCommand) -> Vec<testing::RoutedEvent> {
        match command {
            p2p::Libp2pCommand::FetchSnapshot { peer_id, room, part } => {
                let id = self.claim_reply_id(from);
                if self.is_connected(from, &peer_id) {
                    vec![testing::RoutedEvent {
                        to: peer_id,
                        event: p2p::Libp2pEvent::SnapshotRequest {
                            id,
                            from: from.clone(),
                            room,
                            part,
                        },
                    }]
                } else {
                    vec![testing::RoutedEvent {
                        to: from.clone(),
                        event: p2p::Libp2pEvent::RoomSyncRequestFailed {
                            from: peer_id,
                            request: p2p::SnapshotRequest::Fetch { room, part },
                        },
                    }]
                }
            }
            p2p::Libp2pCommand::FetchReplicaSlice { peer_id, room, index } => {
                let id = self.claim_reply_id(from);
                if self.is_connected(from, &peer_id) {
                    vec![testing::RoutedEvent {
                        to: peer_id,
                        event: p2p::Libp2pEvent::ReplicaSliceRequest {
                            id,
                            from: from.clone(),
                            room,
                            index,
                        },
                    }]
                } else {
                    vec![testing::RoutedEvent {
                        to: from.clone(),
                        event: p2p::Libp2pEvent::RoomSyncRequestFailed {
                            from: peer_id,
                            request: p2p::SnapshotRequest::FetchReplicaSlice { room, index },
                        },
                    }]
                }
            }
            p2p::Libp2pCommand::RespondSnapshot { id, frame } => {
                let Some(requester) = self.pending_snapshot_replies.remove(&id) else {
                    return Vec::new();
                };
                let event = match frame {
                    p2p::SnapshotResponse::Part { room, part, total, data } => {
                        p2p::Libp2pEvent::SnapshotPart {
                            from: from.clone(),
                            room,
                            part,
                            total,
                            data,
                        }
                    }
                    p2p::SnapshotResponse::Missing { room } => p2p::Libp2pEvent::SnapshotMissing {
                        from: from.clone(),
                        room,
                    },
                    p2p::SnapshotResponse::ReplicaSlice { room, index, data } => {
                        p2p::Libp2pEvent::ReplicaSlice {
                            from: from.clone(),
                            room,
                            index,
                            data,
                        }
                    }
                };
                vec![testing::RoutedEvent {
                    to: requester,
                    event,
                }]
            }
            _ => Vec::new(),
        }
    }
    pub(crate) fn fan_out_peer_message(&self, from: &identity::PeerId, command: p2p::Libp2pCommand) -> Vec<testing::RoutedEvent> {
        match command {
            p2p::Libp2pCommand::Publish { topic, data } => {
                let Some(connected) = self.connected_peers.get(from) else {
                    return Vec::new();
                };
                connected
                    .iter()
                    .filter(|peer| *peer != from)
                    .filter(|peer| {
                        self.subscribed_topics
                            .get(*peer)
                            .is_some_and(|topics| topics.contains(&topic))
                    })
                    .map(|to| testing::RoutedEvent {
                        to: to.clone(),
                        event: p2p::Libp2pEvent::Gossip {
                            topic: topic.clone(),
                            from: from.clone(),
                            data: data.clone(),
                        },
                    })
                    .collect()
            }
            p2p::Libp2pCommand::PeerExchange { peer_id, data } => {
                if !self.is_connected(from, &peer_id) {
                    return Vec::new();
                }
                vec![testing::RoutedEvent {
                    to: peer_id,
                    event: p2p::Libp2pEvent::PeerExchange {
                        from: from.clone(),
                        data,
                    },
                }]
            }
            _ => Vec::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::PeerMessageRouter;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::testing;

    fn registered(router: &mut testing::PeerMessageRouter, id: &str) -> identity::PeerId {
        let peer = identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        peer.id
    }

    fn dial(
        router: &mut testing::PeerMessageRouter,
        from: &identity::PeerId,
        to: &identity::PeerId,
    ) {
        router.route(
            &from.clone(),
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: to.clone(),
                addrs: Vec::new(),
            }),
        );
    }

    fn dialed(router: &mut testing::PeerMessageRouter, from: &identity::PeerId, to: &str) {
        let target = identity::Peer {
            id: identity::PeerId::from(to),
            addrs: Vec::new(),
        };
        router.register_peer(&target);
        router.route(&from.clone(), p2p::Libp2pCommand::Dial(target));
    }

    fn subscribe(router: &mut testing::PeerMessageRouter, peer: &identity::PeerId, topic: &str) {
        router.route(
            &peer.clone(),
            p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from(topic),
            },
        );
    }

    fn manifest(prefix: &str) -> p2p::RoomHistoryRecordKind {
        p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from(prefix))
    }

    #[test]
    fn test_usage() {
        let mut router = PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        let events = router.remove_peer(&peer.id);
        assert_eq!(events, Vec::new());
        let events = router.remove_peer(&identity::PeerId::from("unknown"));
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn history_put_then_get_round_trip() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let id = manifest("abc");
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: id.clone(),
                value: vec![1, 2, 3],
                ttl: std::time::Duration::from_secs(90),
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: peer.clone(),
                event: p2p::Libp2pEvent::HistoryRecordStored { id: id.clone() },
            }]
        );
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::GetHistoryRecord { id: id.clone() },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: peer,
                event: p2p::Libp2pEvent::HistoryRecordFound {
                    id,
                    value: vec![1, 2, 3],
                },
            }]
        );
    }

    #[test]
    fn missing_records_report_not_found() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let id = manifest("missing");
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::GetHistoryRecord { id: id.clone() },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: peer,
                event: p2p::Libp2pEvent::HistoryRecordNotFound { id },
            }]
        );
    }

    #[test]
    fn the_store_is_shared_between_peers() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let writer = registered(&mut router, "a");
        let reader = registered(&mut router, "b");
        let id = manifest("shared");
        router.serve_history_command(
            &writer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: id.clone(),
                value: vec![7],
                ttl: std::time::Duration::from_secs(90),
            },
        );
        let events =
            router.serve_history_command(&reader, p2p::Libp2pCommand::GetHistoryRecord { id });
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::HistoryRecordFound { value, .. },
                ..
            }] if value == &vec![7]
        ));
    }

    #[test]
    fn measuring_reports_bytes_by_prefix() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
                value: vec![1, 2, 3],
                ttl: std::time::Duration::from_secs(90),
            },
        );
        router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                    prefix: prefix.clone(),
                    content_hash: p2p::ContentHash::from("h"),
                    chunk: p2p::ChunkIndex(0),
                }),
                value: vec![0; 10],
                ttl: std::time::Duration::from_secs(90),
            },
        );
        let events = router.serve_history_command(&peer, p2p::Libp2pCommand::MeasureHistoryStore);
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: peer,
                event: p2p::Libp2pEvent::HistoryStoreMeasured {
                    bytes_by_prefix: BTreeMap::from([(prefix, p2p::HistoryByteCount(13))]),
                },
            }]
        );
    }

    #[test]
    fn removal_drops_only_the_matching_prefix() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = registered(&mut router, "a");
        for name in ["gone", "kept"] {
            router.serve_history_command(
                &peer,
                p2p::Libp2pCommand::PutHistoryRecord {
                    id: manifest(name),
                    value: vec![1],
                    ttl: std::time::Duration::from_secs(90),
                },
            );
        }
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::RemoveHistoryRecords {
                prefix: p2p::HistoryKeyPrefix::from("gone"),
            },
        );
        assert_eq!(events, Vec::new());
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::GetHistoryRecord {
                id: manifest("gone"),
            },
        );
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::HistoryRecordNotFound { .. },
                ..
            }]
        ));
        let events = router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::GetHistoryRecord {
                id: manifest("kept"),
            },
        );
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::HistoryRecordFound { .. },
                ..
            }]
        ));
    }

    #[test]
    fn snapshot_fetch_then_respond_round_trip() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = registered(&mut router, "a");
        dialed(&mut router, &requester, "b");
        let room = identity::RoomName::from("room-a");
        let events = router.serve_room_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                room: room.clone(),
                part: 0,
            },
        );
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::SnapshotRequest { part: 0, .. },
                ..
            }] if events[0].to == identity::PeerId::from("b")
        ));
        let request_id = match &events[0].event {
            p2p::Libp2pEvent::SnapshotRequest { id, .. } => *id,
            _ => p2p::ReplyId(0),
        };
        let responder = identity::PeerId::from("b");
        let events = router.serve_room_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::Part {
                    room: room.clone(),
                    part: 0,
                    total: 1,
                    data: vec![9],
                },
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::SnapshotPart {
                    from: responder,
                    room,
                    part: 0,
                    total: 1,
                    data: vec![9],
                },
            }]
        );
    }

    #[test]
    fn fetching_from_an_unconnected_peer_fails_for_the_requester() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = registered(&mut router, "a");
        registered(&mut router, "b");
        let room = identity::RoomName::from("room-a");
        let events = router.serve_room_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                room: room.clone(),
                part: 2,
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::RoomSyncRequestFailed {
                    from: identity::PeerId::from("b"),
                    request: p2p::SnapshotRequest::Fetch { room, part: 2 },
                },
            }]
        );
    }

    #[test]
    fn fetching_from_an_unknown_peer_fails_for_the_requester() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = registered(&mut router, "a");
        let room = identity::RoomName::from("room-a");
        let events = router.serve_room_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchReplicaSlice {
                peer_id: identity::PeerId::from("ghost"),
                room: room.clone(),
                index: room_sync::ReplicaSliceIndex(1),
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::RoomSyncRequestFailed {
                    from: identity::PeerId::from("ghost"),
                    request: p2p::SnapshotRequest::FetchReplicaSlice {
                        room,
                        index: room_sync::ReplicaSliceIndex(1),
                    },
                },
            }]
        );
    }

    #[test]
    fn replica_slices_round_trip_through_the_remembered_requester() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = registered(&mut router, "a");
        dialed(&mut router, &requester, "b");
        let room = identity::RoomName::from("room-a");
        let index = room_sync::ReplicaSliceIndex(3);
        let events = router.serve_room_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchReplicaSlice {
                peer_id: identity::PeerId::from("b"),
                room: room.clone(),
                index,
            },
        );
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::ReplicaSliceRequest { .. },
                ..
            }]
        ));
        let request_id = match &events[0].event {
            p2p::Libp2pEvent::ReplicaSliceRequest { id, .. } => *id,
            _ => p2p::ReplyId(0),
        };
        let responder = identity::PeerId::from("b");
        let events = router.serve_room_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::ReplicaSlice {
                    room: room.clone(),
                    index,
                    data: vec![5],
                },
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::ReplicaSlice {
                    from: responder,
                    room,
                    index,
                    data: vec![5],
                },
            }]
        );
    }

    #[test]
    fn missing_snapshots_report_the_responder() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = registered(&mut router, "a");
        dialed(&mut router, &requester, "b");
        let room = identity::RoomName::from("room-a");
        let events = router.serve_room_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                room: room.clone(),
                part: 0,
            },
        );
        let request_id = match &events[0].event {
            p2p::Libp2pEvent::SnapshotRequest { id, .. } => *id,
            _ => p2p::ReplyId(0),
        };
        let responder = identity::PeerId::from("b");
        let events = router.serve_room_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::Missing { room: room.clone() },
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::SnapshotMissing {
                    from: responder,
                    room,
                },
            }]
        );
    }

    #[test]
    fn replies_to_unknown_ids_are_dropped() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let responder = registered(&mut router, "b");
        let events = router.serve_room_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: p2p::ReplyId(99),
                frame: p2p::SnapshotResponse::Missing {
                    room: identity::RoomName::from("room-a"),
                },
            },
        );
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn publish_reaches_only_subscribed_connections() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let subscriber = registered(&mut router, "b");
        let stranger = registered(&mut router, "c");
        dial(&mut router, &publisher, &subscriber);
        dial(&mut router, &publisher, &stranger);
        subscribe(&mut router, &subscriber, "news");
        let topic = identity::Topic::from("news");
        let events = router.fan_out_peer_message(
            &publisher,
            p2p::Libp2pCommand::Publish {
                topic: topic.clone(),
                data: vec![1, 2],
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: subscriber,
                event: p2p::Libp2pEvent::Gossip {
                    topic,
                    from: publisher,
                    data: vec![1, 2],
                },
            }]
        );
    }

    #[test]
    fn unsubscribed_connected_peers_get_no_gossip() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let listener = registered(&mut router, "b");
        dial(&mut router, &publisher, &listener);
        let events = router.fan_out_peer_message(
            &publisher,
            p2p::Libp2pCommand::Publish {
                topic: identity::Topic::from("news"),
                data: vec![1],
            },
        );
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn unconnected_subscribers_get_no_gossip() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let subscriber = registered(&mut router, "b");
        subscribe(&mut router, &subscriber, "news");
        let events = router.fan_out_peer_message(
            &publisher,
            p2p::Libp2pCommand::Publish {
                topic: identity::Topic::from("news"),
                data: vec![1],
            },
        );
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn peer_exchange_reaches_only_connected_peers() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let sender = registered(&mut router, "a");
        let friend = registered(&mut router, "b");
        let stranger = registered(&mut router, "c");
        dial(&mut router, &sender, &friend);
        let events = router.fan_out_peer_message(
            &sender,
            p2p::Libp2pCommand::PeerExchange {
                peer_id: friend.clone(),
                data: vec![3],
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: friend,
                event: p2p::Libp2pEvent::PeerExchange {
                    from: sender.clone(),
                    data: vec![3],
                },
            }]
        );
        let events = router.fan_out_peer_message(
            &sender,
            p2p::Libp2pCommand::PeerExchange {
                peer_id: stranger,
                data: vec![3],
            },
        );
        assert_eq!(events, Vec::new());
    }
}
