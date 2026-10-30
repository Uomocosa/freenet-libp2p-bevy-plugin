use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, e2e_report, p2p};

use crate::script_runner::StarboardScriptRunner;
use crate::starboard;

pub fn record_peer_run_result(
    time: Res<Time>,
    status: Res<discovery::Status>,
    canvas: Res<starboard::canvas::Canvas>,
    runner: Res<StarboardScriptRunner>,
    channel: Res<p2p::NetEventChannel>,
    mut subscription: Local<Option<tokio::sync::broadcast::Receiver<p2p::NetEvent>>>,
    mut result: ResMut<e2e_report::PeerRunResult>,
) {
    let time = time.into_inner();
    let status = status.into_inner();
    let canvas = canvas.into_inner();
    let runner = runner.into_inner();
    let channel = channel.into_inner();
    if subscription.is_none() {
        *subscription = Some(channel.subscribe());
    }
    if let Some(rx) = subscription.as_mut() {
        loop {
            match rx.try_recv() {
                Ok(p2p::NetEvent::Libp2pConnectionEstablished { peer_id, kind }) => {
                    result
                        .connection_kinds
                        .entry(peer_id)
                        .or_default()
                        .push(kind);
                }
                Ok(_) => {}
                Err(
                    tokio::sync::broadcast::error::TryRecvError::Empty
                    | tokio::sync::broadcast::error::TryRecvError::Closed,
                ) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(skipped)) => {
                    tracing::warn!("result tap lagged, skipped {skipped} net events");
                }
            }
        }
    }
    if let Some(room) = status.room.as_ref() {
        for peer in room.members.keys() {
            result.members_seen.insert(peer.clone());
        }
        if let Some(me) = status.self_peer_id.as_ref() {
            result.members_seen.insert(me.clone());
        }
    }
    if status.is_lobby_attached && result.freenet.lobby_attached_after_secs.is_none() {
        result.freenet.lobby_attached_after_secs = Some(time.elapsed_secs_f64());
    }
    result.freenet.freenet_peers_seen = result
        .freenet
        .freenet_peers_seen
        .max(status.freenet_peers.len());
    result.steps.clone_from(&runner.finished_step_timings);
    if starboard::room::is_in_room(status) {
        result.canvas_hash = Some(starboard::canvas::hash_canvas(canvas));
        result.tiles = canvas.tiles.iter().filter(|tile| tile.is_some()).count();
    }
    result.error = runner
        .finished_step_timings
        .iter()
        .find_map(|timing| timing.failure.clone());
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, e2e_report, identity, p2p};

    use super::record_peer_run_result;
    use crate::script_runner::StarboardScriptRunner;
    use crate::shared::script::timed_script_step::parse_timed_script_step;
    use crate::starboard;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.init_resource::<discovery::Status>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<p2p::NetEventChannel>();
        let steps = ["sleep:1s"]
            .iter()
            .map(|raw| {
                parse_timed_script_step::<crate::script::StarboardScriptStep>(raw).expect("step")
            })
            .collect();
        app.insert_resource(StarboardScriptRunner::new(steps));
        app.insert_resource(e2e_report::PeerRunResult {
            username: "p1".to_string(),
            ..e2e_report::PeerRunResult::default()
        });
        app.add_systems(Update, record_peer_run_result);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.update();
        let result = app.world().resource::<e2e_report::PeerRunResult>();
        assert_eq!(result.username, "p1");
        assert_eq!(result.steps.len(), 0);
    }

    #[test]
    fn connection_events_are_recorded() {
        let mut app = app();
        app.update();
        app.world().resource::<p2p::NetEventChannel>().send(
            p2p::NetEvent::Libp2pConnectionEstablished {
                peer_id: identity::PeerId::from("peer-a"),
                kind: p2p::ConnectionKind::Tcp,
            },
        );
        app.update();
        let result = app.world().resource::<e2e_report::PeerRunResult>();
        assert_eq!(
            result
                .connection_kinds
                .get(&identity::PeerId::from("peer-a")),
            Some(&vec![p2p::ConnectionKind::Tcp])
        );
    }

    #[test]
    fn lobby_attach_time_is_set_once() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(2));
        app.world_mut()
            .resource_mut::<discovery::Status>()
            .is_lobby_attached = true;
        app.update();
        let first = app
            .world()
            .resource::<e2e_report::PeerRunResult>()
            .freenet
            .lobby_attached_after_secs;
        assert!(first.is_some());
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(5));
        app.update();
        let second = app
            .world()
            .resource::<e2e_report::PeerRunResult>()
            .freenet
            .lobby_attached_after_secs;
        assert_eq!(first, second);
    }

    #[test]
    fn members_seen_counts_self_with_room_members() {
        let mut app = app();
        let me = identity::PeerId::from("me");
        let other = identity::PeerId::from("peer-a");
        let mut members = discovery::Members::new();
        members.insert(
            other.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: other.clone(),
                    addrs: Vec::new(),
                },
                display_name: None,
                updated_at: discovery::UnixTime::from(0),
                membership: discovery::RoomMembership::Occupant,
            },
        );
        {
            let mut status = app.world_mut().resource_mut::<discovery::Status>();
            status.room = Some(discovery::JoinedRoom {
                name: identity::RoomName::from("room-1"),
                created_at: discovery::UnixTime::from(1),
                phase: discovery::JoinPhase::Ready,
                members,
                entry: discovery::RoomEntry::Joined,
            });
            status.self_peer_id = Some(me.clone());
        }
        app.update();
        let result = app.world().resource::<e2e_report::PeerRunResult>();
        assert!(result.members_seen.contains(&me));
        assert!(result.members_seen.contains(&other));
        assert_eq!(result.members_seen.len(), 2);
    }
}
