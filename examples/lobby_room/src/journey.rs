//! End-to-end UI preview of the whole `lobby_room` passage, with a fake discovery backend:
//! app start (plugin connecting screen) -> freenet node reachable + lobby listed -> click Join
//! on a room -> peers arrive one by one -> hover the leave door -> click it, answer No
//! -> press Esc, answer Yes -> back in the lobby. Only `DiscoveryState` is faked; the click
//! goes through the plugin's real `ui::discovery::DefaultUiPlugin` button path and the
//! real `discovery::Command`.

#[cfg(test)]
mod tests {
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::ui::Pressed;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::testing;
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{discovery, identity, ui};

    use crate::overlay;
    use examples_shared::status;

    const JOIN_ROOM: &str = "gamma";
    const JOURNEY_FPS: u32 = 30;
    const NODE_REACHABLE_AT: u32 = 20;
    const CLICK_JOIN_AT: u32 = 40;
    const FIRST_PEER_AT: u32 = 55;
    const PEER_EVERY: u32 = 15;
    const HOVER_DOOR_AT: u32 = 100;
    const CLICK_DOOR_AT: u32 = 130;
    const ANSWER_NO_AT: u32 = 155;
    const PRESS_ESC_AT: u32 = 175;
    const ANSWER_YES_AT: u32 = 200;
    const END_AT: u32 = 230;
    const PEERS: [(&str, &str); 3] = [
        ("12D3KooWbob000Kd2a", "bob"),
        ("12D3KooWcarol0Zm9c", "carol"),
        ("12D3KooWdave00Qx7f", "dave"),
    ];

    /// Journey frames (at `JOURNEY_FPS` of virtual time) already acted on; absent until the
    /// timeline kicks it off. Keyed on `Time`, not on updates: the preview pumps extra
    /// zero-delta updates while it captures each frame.
    #[derive(Resource, Default)]
    struct JourneyClock {
        started_at: Option<f32>,
        frame: u32,
    }

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build(),
        );
        app.insert_resource(status::LocalUser(identity::Username::from("alice")));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins((ui::discovery::DefaultUiPlugin, overlay::Overlay));
        app.add_systems(
            Update,
            (
                drive_journey
                    .run_if(resource_exists::<JourneyClock>)
                    .before(ui::discovery::bevy_systems::handle_join)
                    .before(ui::discovery::bevy_systems::request_leave),
                fake_backend,
            ),
        );
    }

    /// # needed helper: scripted network + user timeline, keyed on the frame counter
    fn drive_journey(world: &mut World) {
        release_presses(world);
        let elapsed = world.resource::<Time>().elapsed_secs();
        let mut clock = world.resource_mut::<JourneyClock>();
        let started_at = *clock.started_at.get_or_insert(elapsed);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            clippy::as_conversions
        )]
        let frame = ((elapsed - started_at) * JOURNEY_FPS as f32).round() as u32;
        let is_new_frame = frame != clock.frame;
        clock.frame = frame;
        if (HOVER_DOOR_AT..CLICK_DOOR_AT).contains(&frame) {
            hover_marker::<ui::discovery::LeaveDoor>(world);
        }
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Escape);
        keys.clear();
        if !is_new_frame {
            return;
        }
        if frame == NODE_REACHABLE_AT {
            node_reachable(world);
            list_lobby(world);
        }
        if frame == CLICK_JOIN_AT {
            click_join(world);
        }
        let arrived = frame
            .checked_sub(FIRST_PEER_AT)
            .filter(|since| since % PEER_EVERY == 0)
            .map(|since| {
                usize::try_from(since.checked_div(PEER_EVERY).unwrap_or(0)).unwrap_or(usize::MAX)
            });
        if let Some(index) = arrived.filter(|index| *index < PEERS.len()) {
            add_peer(world, index);
        }
        match frame {
            CLICK_DOOR_AT => press_marker::<ui::discovery::LeaveDoor>(world),
            ANSWER_NO_AT => press_marker::<ui::discovery::LeaveNoButton>(world),
            PRESS_ESC_AT => {
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Escape);
            }
            ANSWER_YES_AT => press_marker::<ui::discovery::LeaveYesButton>(world),
            _ => {}
        }
    }

    /// # needed helper: a scripted press lasts one frame, then the "mouse" is released
    fn release_presses(world: &mut World) {
        let pressed: Vec<Entity> = world
            .query_filtered::<Entity, With<Pressed>>()
            .iter(world)
            .collect();
        for entity in pressed {
            world.entity_mut(entity).remove::<Pressed>();
        }
    }

    /// # needed helper: drives a button's real hover state, as the cursor would
    fn hover_marker<Marker: Component>(world: &mut World) {
        let targets: Vec<Entity> = world
            .query_filtered::<Entity, With<Marker>>()
            .iter(world)
            .collect();
        for entity in targets {
            world.entity_mut(entity).insert(Hovered(true));
        }
    }

    /// # needed helper: drives a button's real press, as the cursor would
    fn press_marker<Marker: Component>(world: &mut World) {
        let targets: Vec<Entity> = world
            .query_filtered::<Entity, With<Marker>>()
            .iter(world)
            .collect();
        for entity in targets {
            world.entity_mut(entity).insert(Hovered(true));
            world.entity_mut(entity).insert(Pressed);
        }
    }

    /// # needed helper: stands in for the discovery task — `Join` gives an empty room, `Leave`
    /// drops it
    fn fake_backend(
        mut commands: MessageReader<discovery::Command>,
        mut state: ResMut<DiscoveryState>,
    ) {
        for command in commands.read() {
            match command {
                discovery::Command::Join(name) => {
                    state.room = Some(discovery::JoinedRoom {
                        name: name.clone(),
                        created_at: time::UnixTime::EPOCH,
                        phase: discovery::JoinPhase::Ready,
                        members: discovery::Members::new(),
                        how_entered: discovery::HowRoomWasEntered::Created,
                    });
                }
                discovery::Command::Leave => state.room = None,
                _ => {}
            }
        }
    }

    fn node_reachable(world: &mut World) {
        let mut state = world.resource_mut::<DiscoveryState>();
        state.freenet_peers = (0..1)
            .map(|index| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(format!("gateway-{index}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], 31337)),
            })
            .collect();
        state.is_lobby_fetched = true;
    }

    fn list_lobby(world: &mut World) {
        let mut state = world.resource_mut::<DiscoveryState>();
        for (name, room_size) in [("alpha", 8), ("beta", 4), (JOIN_ROOM, 8)] {
            state.lobby.insert(
                identity::RoomName::from(name),
                discovery::Room {
                    room_size: discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
    }

    /// # needed helper: presses the room's real Join button, so the default UI emits the command
    fn click_join(world: &mut World) {
        let targets: Vec<Entity> = world
            .query_filtered::<Entity, With<ui::discovery::RoomButton>>()
            .iter(world)
            .collect();
        for entity in targets {
            let is_target = world
                .get::<ui::discovery::RoomButton>(entity)
                .is_some_and(|button| button.as_str() == JOIN_ROOM);
            if is_target {
                world.entity_mut(entity).insert(Hovered(true));
                world.entity_mut(entity).insert(Pressed);
            }
        }
    }

    fn add_peer(world: &mut World, index: usize) {
        let Some((peer, name)) = PEERS.get(index) else {
            return;
        };
        let mut state = world.resource_mut::<DiscoveryState>();
        let Some(room) = state.room.as_mut() else {
            return;
        };
        room.members.insert(
            identity::PeerId::from(*peer),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from(*peer),
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::EPOCH,
                display_name: Some(identity::Username::from(*name)),
                membership: discovery::RoomMembership::Occupant,
            },
        );
    }

    fn start_journey(world: &mut World) {
        world.init_resource::<JourneyClock>();
    }

    const fn preview_noop(_world: &mut World) {}

    fn lobby_listed(world: &mut World) {
        node_reachable(world);
        list_lobby(world);
    }

    fn room_full(world: &mut World) {
        lobby_listed(world);
        world.resource_mut::<DiscoveryState>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from(JOIN_ROOM),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        for index in 0..PEERS.len() {
            add_peer(world, index);
        }
    }

    fn leave_prompt(world: &mut World) {
        room_full(world);
        world
            .resource_mut::<NextState<ui::discovery::Screen>>()
            .set(ui::discovery::Screen::InRoom);
        world.run_schedule(StateTransition);
        world
            .resource_mut::<NextState<ui::discovery::LeaveDialog>>()
            .set(ui::discovery::LeaveDialog::Open);
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("lobby_journey"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("app started, connecting"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("lobby listed"),
                    apply: lobby_listed,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("joined with peers"),
                    apply: room_full,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("leave prompt"),
                    apply: leave_prompt,
                },
            ],
            timeline,
        }
    }

    #[test]
    fn lobby_journey_ui_png_preview() {
        scene(None).run(&config()).expect("preview");
    }

    #[test]
    fn lobby_journey_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("start_join_peers_leave"),
            frames: END_AT,
            fps: JOURNEY_FPS,
            apply: start_journey,
        };
        scene(Some(timeline)).run(&config()).expect("preview");
    }
}
