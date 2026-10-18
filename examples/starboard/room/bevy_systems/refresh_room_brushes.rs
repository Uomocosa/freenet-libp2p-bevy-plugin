use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, p2p};

use crate::starboard;

pub fn refresh_room_brushes(
    time: Res<Time>,
    ready: Option<Res<p2p::ReadyChannel>>,
    status: Option<Res<discovery::Status>>,
    mut timer: ResMut<starboard::room::BrushRefreshTimer>,
    mut brushes: ResMut<starboard::room::RoomBrushes>,
    mut own: ResMut<starboard::input::OwnBrush>,
) {
    let time = time.into_inner();
    if !timer.0.tick(time.delta()).is_finished() {
        return;
    }
    let status = status.map(Res::into_inner);
    let ready = ready.map(Res::into_inner);
    let Some(joined) = status.as_ref().and_then(|status| status.room.as_ref()) else {
        return;
    };
    let now = discovery::UnixTime::now();
    let next = starboard::identity::room_brushes(joined.members.keys(), joined.created_at, now);
    if next != brushes.0 {
        brushes.0 = next;
    }
    let Some(me) = ready.and_then(|ready| ready.ready_rx.borrow().clone()) else {
        return;
    };
    if let Some(brush) = brushes.0.get(&me.id) {
        own.shape = brush.shape;
        own.color = brush.color;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity, p2p};

    use super::refresh_room_brushes;
    use crate::starboard;

    fn joined_status() -> discovery::Status {
        let mut members = discovery::Members::new();
        for peer in ["peer-a", "peer-b"] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: None,
                    },
                    status: discovery::LinkStatus::Connected,
                },
            );
        }
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(1),
                phase: discovery::RoomPhase::Ready,
                members,
            }),
            ..discovery::Status::default()
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(joined_status());
        app.insert_resource(starboard::room::BrushRefreshTimer(Timer::from_seconds(
            0.0,
            TimerMode::Repeating,
        )));
        app.init_resource::<starboard::room::RoomBrushes>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.init_resource::<p2p::ReadyChannel>();
        app.add_systems(Update, refresh_room_brushes);

        let _ = app
            .world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(identity::Peer {
                id: identity::PeerId::from("peer-a"),
                addrs: Vec::new(),
            }));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.update();

        let brushes = app.world().resource::<starboard::room::RoomBrushes>();
        assert_eq!(brushes.len(), 2);
        let mine = brushes
            .get(&identity::PeerId::from("peer-a"))
            .copied()
            .expect("own brush assigned");
        let own = app.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(own.shape, mine.shape);
        assert_eq!(own.color, mine.color);
    }
}
