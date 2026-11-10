use bevy::prelude::*;

use crate::discovery;
use core_discovery::state_machine::DiscoveryState;
use discovery::{JoiningRoomRoot, Screen};
use freenet_libp2p_bevy_core::discovery as core_discovery;

pub fn spawn_joining_room(mut commands: Commands, state: Option<Res<DiscoveryState>>) {
    let label = state
        .and_then(|state| {
            state
                .into_inner()
                .room
                .as_ref()
                .map(|room| format!("Joining room {}...", room.name.as_str()))
        })
        .unwrap_or_else(|| String::from("Joining room..."));
    commands.spawn((
        JoiningRoomRoot,
        DespawnOnExit(Screen::JoiningRoom),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        children![(Text::new(label), TextColor(Color::WHITE))],
    ));
}
#[cfg(test)]
mod tests {
    use crate::discovery;
    use bevy::prelude::*;

    use super::spawn_joining_room;
    use core_discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_core::discovery as core_discovery;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::testing;
    use freenet_libp2p_bevy_core::time;

    fn snapshot_in(room: &str) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(core_discovery::JoinedRoom {
                name: identity::RoomName::from(room),
                created_at: time::UnixTime::EPOCH,
                phase: core_discovery::JoinPhase::Joining {
                    since: time::UnixTime::EPOCH,
                },
                members: core_discovery::Members::new(),
                how_entered: core_discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn label(app: &mut App) -> String {
        let mut texts = app.world_mut().query::<&Text>();
        texts.iter(app.world()).map(|text| text.as_str()).collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(snapshot_in("alpha"));
        app.add_systems(Startup, spawn_joining_room);
        app.update();
        let mut roots = app
            .world_mut()
            .query_filtered::<(), With<discovery::JoiningRoomRoot>>();
        assert_eq!(roots.iter(app.world()).count(), 1);
        assert_eq!(label(&mut app), "Joining room alpha...");
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(snapshot_in("alpha"));
        app.add_systems(
            Startup,
            (
                spawn_joining_room,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_joining_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_joining_room"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("joining room"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
