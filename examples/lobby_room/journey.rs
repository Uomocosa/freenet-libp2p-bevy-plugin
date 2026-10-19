//! End-to-end UI preview of the whole `lobby_room` passage, with a fake discovery backend:
//! app start (plugin connecting screen) -> freenet ring ready + lobby listed -> click Join
//! on a room -> peers arrive one by one -> hover the leave door -> click it, answer No
//! -> press Esc, answer Yes -> back in the lobby. Only `discovery::Status` is faked; the click
//! goes through the plugin's real `discovery::ui::DefaultUiPlugin` button path and the
//! real `discovery::Command`.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use crate::{overlay, shared::status};

    const JOIN_ROOM: &str = "gamma";
    const JOURNEY_FPS: u32 = 30;
    const RING_READY_AT: u32 = 20;
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
        app.insert_resource(discovery::Status::default());
        app.insert_resource(status::LocalUser(identity::Username::from("alice")));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins((discovery::ui::DefaultUiPlugin, overlay::Overlay));
        app.add_systems(
            Update,
            (
                drive_journey
                    .run_if(resource_exists::<JourneyClock>)
                    .before(discovery::ui::bevy_systems::handle_join)
                    .before(discovery::ui::bevy_systems::request_leave),
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
        let frame = ((elapsed - started_at) * JOURNEY_FPS as f32).round() as u32;
        let is_new_frame = frame != clock.frame;
        clock.frame = frame;
        if (HOVER_DOOR_AT..CLICK_DOOR_AT).contains(&frame) {
            press::<discovery::ui::LeaveDoor>(world, Interaction::Hovered);
        }
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Escape);
        keys.clear();
        if !is_new_frame {
            return;
        }
        if frame == RING_READY_AT {
            ring_ready(world);
            list_lobby(world);
        }
        if frame == CLICK_JOIN_AT {
            click_join(world);
        }
        let arrived = frame
            .checked_sub(FIRST_PEER_AT)
            .filter(|since| since % PEER_EVERY == 0)
            .map(|since| (since / PEER_EVERY) as usize);
        if let Some(index) = arrived.filter(|index| *index < PEERS.len()) {
            add_peer(world, index);
        }
        match frame {
            CLICK_DOOR_AT => press::<discovery::ui::LeaveDoor>(world, Interaction::Pressed),
            ANSWER_NO_AT => press::<discovery::ui::LeaveNoButton>(world, Interaction::Pressed),
            PRESS_ESC_AT => {
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Escape);
            }
            ANSWER_YES_AT => press::<discovery::ui::LeaveYesButton>(world, Interaction::Pressed),
            _ => {}
        }
    }

    /// # needed helper: a scripted press lasts one frame, then the "mouse" is released
    fn release_presses(world: &mut World) {
        let mut buttons = world.query::<&mut Interaction>();
        for mut interaction in buttons.iter_mut(world) {
            if *interaction == Interaction::Pressed {
                *interaction = Interaction::None;
            }
        }
    }

    /// # needed helper: drives a button's real `Interaction`, as the cursor would
    fn press<Marker: Component>(world: &mut World, state: Interaction) {
        let mut buttons = world.query_filtered::<&mut Interaction, With<Marker>>();
        for mut interaction in buttons.iter_mut(world) {
            *interaction = state;
        }
    }

    /// # needed helper: stands in for the discovery task — `Join` gives an empty room, `Leave`
    /// drops it
    fn fake_backend(
        mut commands: MessageReader<discovery::Command>,
        mut status: ResMut<discovery::Status>,
    ) {
        for command in commands.read() {
            match command {
                discovery::Command::Join(name) => {
                    status.room = Some(discovery::JoinedRoom {
                        name: name.clone(),
                        created_at: discovery::UnixTime::from_secs(0),
                        phase: discovery::RoomPhase::Ready,
                        members: discovery::Members::new(),
                    });
                }
                discovery::Command::Leave => status.room = None,
                _ => {}
            }
        }
    }

    fn ring_ready(world: &mut World) {
        let mut status = world.resource_mut::<discovery::Status>();
        status.freenet_peers = (0..1)
            .map(|index| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(format!("gateway-{index}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], 31337)),
            })
            .collect();
        status.is_lobby_attached = true;
    }

    fn list_lobby(world: &mut World) {
        let mut status = world.resource_mut::<discovery::Status>();
        for (name, room_size) in [("alpha", 8), ("beta", 4), (JOIN_ROOM, 8)] {
            status.lobby.insert(
                identity::RoomName::from(name),
                discovery::Room {
                    room_size: discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: discovery::UnixTime::from_secs(0),
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
    }

    /// # needed helper: presses the room's real Join button, so the default UI emits the command
    fn click_join(world: &mut World) {
        let mut buttons = world.query::<(&discovery::ui::RoomButton, &mut Interaction)>();
        for (button, mut interaction) in buttons.iter_mut(world) {
            if button.as_str() == JOIN_ROOM {
                *interaction = Interaction::Pressed;
            }
        }
    }

    fn add_peer(world: &mut World, index: usize) {
        let Some((peer, name)) = PEERS.get(index) else {
            return;
        };
        let mut status = world.resource_mut::<discovery::Status>();
        let Some(room) = status.room.as_mut() else {
            return;
        };
        room.members.insert(
            identity::PeerId::from(*peer),
            discovery::Member {
                heartbeat: discovery::Heartbeat {
                    addrs: Vec::new(),
                    updated_at: discovery::UnixTime::from_secs(0),
                    display_name: Some(identity::Username::from(*name)),
                    presence: discovery::Presence::InRoom,
                },
                status: discovery::LinkStatus::Connected,
            },
        );
    }

    fn start_journey(world: &mut World) {
        world.init_resource::<JourneyClock>();
    }

    const fn preview_noop(_world: &mut World) {}

    fn lobby_listed(world: &mut World) {
        ring_ready(world);
        list_lobby(world);
    }

    fn room_full(world: &mut World) {
        lobby_listed(world);
        world.resource_mut::<discovery::Status>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from(JOIN_ROOM),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
        for index in 0..PEERS.len() {
            add_peer(world, index);
        }
    }

    fn leave_prompt(world: &mut World) {
        room_full(world);
        world
            .resource_mut::<NextState<discovery::ui::Screen>>()
            .set(discovery::ui::Screen::InRoom);
        world.run_schedule(StateTransition);
        world
            .resource_mut::<NextState<discovery::ui::LeaveDialog>>()
            .set(discovery::ui::LeaveDialog::Open);
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
        lele_bevy_preview::run(&scene(None), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn lobby_journey_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("start_join_peers_leave"),
            frames: END_AT,
            fps: JOURNEY_FPS,
            apply: start_journey,
        };
        lele_bevy_preview::run(&scene(Some(timeline)), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }
}
