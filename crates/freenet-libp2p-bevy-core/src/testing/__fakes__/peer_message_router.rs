use std::collections::{BTreeMap, BTreeSet};

use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::p2p;
use crate::testing;

pub struct PeerMessageRouter {
    pub(crate) game: identity::GameId,
    pub(crate) registered_peers: BTreeSet<identity::PeerId>,
    pub(crate) connected_peers: BTreeMap<identity::PeerId, BTreeSet<identity::PeerId>>,
    pub(crate) subscribed_topics: BTreeMap<identity::PeerId, BTreeSet<identity::Topic>>,
    pub(crate) joined_zones: BTreeMap<identity::PeerId, identity::ZoneName>,
    pub(crate) watched_zones: BTreeMap<identity::PeerId, BTreeMap<identity::ZoneName, u32>>,
    pub(crate) history_values: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) history_kinds: BTreeMap<Vec<u8>, p2p::ZoneHistoryRecordKind>,
    pub(crate) pending_snapshot_replies: BTreeMap<p2p::ReplyId, identity::PeerId>,
    pub(crate) next_reply_id: u64,
}

impl PeerMessageRouter {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            registered_peers: BTreeSet::new(),
            connected_peers: BTreeMap::new(),
            subscribed_topics: BTreeMap::new(),
            joined_zones: BTreeMap::new(),
            watched_zones: BTreeMap::new(),
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
    pub fn route_joined_zone_bytes(
        &mut self,
        from: &identity::PeerId,
        data: Vec<u8>,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub fn route_specific_zone_bytes(
        &mut self,
        from: &identity::PeerId,
        zone: &identity::ZoneName,
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
    pub(crate) fn is_connected(&self, first: &identity::PeerId, second: &identity::PeerId) -> bool {
    }
    pub(crate) fn serve_history_command(
        &mut self,
        from: &identity::PeerId,
        command: p2p::Libp2pCommand,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub(crate) fn serve_replica_sync_command(
        &mut self,
        from: &identity::PeerId,
        command: p2p::Libp2pCommand,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub(crate) fn fan_out_peer_message(
        &self,
        from: &identity::PeerId,
        command: p2p::Libp2pCommand,
    ) -> Vec<testing::RoutedEvent> {
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::PeerMessageRouter;
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::testing;

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
        let peer = testing::registered_peer(&mut router, "a");
        let id = testing::manifest("abc");
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
        let peer = testing::registered_peer(&mut router, "a");
        let id = testing::manifest("missing");
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
        let writer = testing::registered_peer(&mut router, "a");
        let reader = testing::registered_peer(&mut router, "b");
        let id = testing::manifest("shared");
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
        let peer = testing::registered_peer(&mut router, "a");
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
                value: vec![1, 2, 3],
                ttl: std::time::Duration::from_secs(90),
            },
        );
        router.serve_history_command(
            &peer,
            p2p::Libp2pCommand::PutHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
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
        let peer = testing::registered_peer(&mut router, "a");
        for name in ["gone", "kept"] {
            router.serve_history_command(
                &peer,
                p2p::Libp2pCommand::PutHistoryRecord {
                    id: testing::manifest(name),
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
                id: testing::manifest("gone"),
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
                id: testing::manifest("kept"),
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
        let requester = testing::registered_peer(&mut router, "a");
        let _ = testing::dialed(&mut router, &requester, "b");
        let zone = identity::ZoneName::from("zone-a");
        let events = router.serve_replica_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                zone: zone.clone(),
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
        let events = router.serve_replica_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::Part {
                    zone: zone.clone(),
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
                    zone,
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
        let requester = testing::registered_peer(&mut router, "a");
        let _ = testing::registered_peer(&mut router, "b");
        let zone = identity::ZoneName::from("zone-a");
        let events = router.serve_replica_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                zone: zone.clone(),
                part: 2,
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::ReplicaSyncRequestFailed {
                    from: identity::PeerId::from("b"),
                    request: p2p::SnapshotRequest::Fetch { zone, part: 2 },
                },
            }]
        );
    }

    #[test]
    fn fetching_from_an_unknown_peer_fails_for_the_requester() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = testing::registered_peer(&mut router, "a");
        let zone = identity::ZoneName::from("zone-a");
        let events = router.serve_replica_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchReplicaSlice {
                peer_id: identity::PeerId::from("ghost"),
                zone: zone.clone(),
                index: replica_sync::ReplicaSliceIndex(1),
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::ReplicaSyncRequestFailed {
                    from: identity::PeerId::from("ghost"),
                    request: p2p::SnapshotRequest::FetchReplicaSlice {
                        zone,
                        index: replica_sync::ReplicaSliceIndex(1),
                    },
                },
            }]
        );
    }

    #[test]
    fn replica_slices_round_trip_through_the_remembered_requester() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = testing::registered_peer(&mut router, "a");
        let _ = testing::dialed(&mut router, &requester, "b");
        let zone = identity::ZoneName::from("zone-a");
        let index = replica_sync::ReplicaSliceIndex(3);
        let events = router.serve_replica_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchReplicaSlice {
                peer_id: identity::PeerId::from("b"),
                zone: zone.clone(),
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
        let events = router.serve_replica_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::ReplicaSlice {
                    zone: zone.clone(),
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
                    zone,
                    index,
                    data: vec![5],
                },
            }]
        );
    }

    #[test]
    fn missing_snapshots_report_the_responder() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = testing::registered_peer(&mut router, "a");
        let _ = testing::dialed(&mut router, &requester, "b");
        let zone = identity::ZoneName::from("zone-a");
        let events = router.serve_replica_sync_command(
            &requester,
            p2p::Libp2pCommand::FetchSnapshot {
                peer_id: identity::PeerId::from("b"),
                zone: zone.clone(),
                part: 0,
            },
        );
        let request_id = match &events[0].event {
            p2p::Libp2pEvent::SnapshotRequest { id, .. } => *id,
            _ => p2p::ReplyId(0),
        };
        let responder = identity::PeerId::from("b");
        let events = router.serve_replica_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: request_id,
                frame: p2p::SnapshotResponse::Missing { zone: zone.clone() },
            },
        );
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: requester,
                event: p2p::Libp2pEvent::SnapshotMissing {
                    from: responder,
                    zone,
                },
            }]
        );
    }

    #[test]
    fn replies_to_unknown_ids_are_dropped() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let responder = testing::registered_peer(&mut router, "b");
        let events = router.serve_replica_sync_command(
            &responder,
            p2p::Libp2pCommand::RespondSnapshot {
                id: p2p::ReplyId(99),
                frame: p2p::SnapshotResponse::Missing {
                    zone: identity::ZoneName::from("zone-a"),
                },
            },
        );
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn publish_reaches_only_subscribed_connections() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = testing::registered_peer(&mut router, "a");
        let subscriber = testing::registered_peer(&mut router, "b");
        let stranger = testing::registered_peer(&mut router, "c");
        let _ = testing::dial(&mut router, &publisher, &subscriber);
        let _ = testing::dial(&mut router, &publisher, &stranger);
        let _ = testing::subscribe(&mut router, &subscriber, "news");
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
        let publisher = testing::registered_peer(&mut router, "a");
        let listener = testing::registered_peer(&mut router, "b");
        let _ = testing::dial(&mut router, &publisher, &listener);
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
        let publisher = testing::registered_peer(&mut router, "a");
        let subscriber = testing::registered_peer(&mut router, "b");
        let _ = testing::subscribe(&mut router, &subscriber, "news");
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
        let sender = testing::registered_peer(&mut router, "a");
        let friend = testing::registered_peer(&mut router, "b");
        let stranger = testing::registered_peer(&mut router, "c");
        let _ = testing::dial(&mut router, &sender, &friend);
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
