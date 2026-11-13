use crate::identity;
use crate::p2p;
use crate::testing;

pub fn serve_room_sync_command(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    command: p2p::Libp2pCommand,
) -> Vec<testing::RoutedEvent> {
    match command {
        p2p::Libp2pCommand::FetchSnapshot {
            peer_id,
            room,
            part,
        } => {
            let id = router.claim_reply_id(from);
            if router.is_connected(from, &peer_id) {
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
        p2p::Libp2pCommand::FetchReplicaSlice {
            peer_id,
            room,
            index,
        } => {
            let id = router.claim_reply_id(from);
            if router.is_connected(from, &peer_id) {
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
            let Some(requester) = router.pending_snapshot_replies.remove(&id) else {
                return Vec::new();
            };
            let event = match frame {
                p2p::SnapshotResponse::Part {
                    room,
                    part,
                    total,
                    data,
                } => p2p::Libp2pEvent::SnapshotPart {
                    from: from.clone(),
                    room,
                    part,
                    total,
                    data,
                },
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

#[cfg(test)]
mod tests {
    use super::serve_room_sync_command;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let from = identity::PeerId::from("a");
        let command = p2p::Libp2pCommand::FetchReplicaSlice {
            peer_id: identity::PeerId::from("ghost"),
            room: identity::RoomName::from("room-1"),
            index: room_sync::ReplicaSliceIndex(1),
        };
        let events = serve_room_sync_command(&mut router, &from, command);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].to, from);
        assert_eq!(
            events[0].event,
            p2p::Libp2pEvent::RoomSyncRequestFailed {
                from: identity::PeerId::from("ghost"),
                request: p2p::SnapshotRequest::FetchReplicaSlice {
                    room: identity::RoomName::from("room-1"),
                    index: room_sync::ReplicaSliceIndex(1),
                },
            }
        );
    }
}
