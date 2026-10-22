use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{JoiningRoomRoot, Screen};

pub fn spawn_joining_room(mut commands: Commands, status: Res<discovery::Status>) {
    let status = status.into_inner();
    let label = status.room.as_ref().map_or_else(
        || String::from("Joining room..."),
        |room| format!("Joining room {}...", room.name.as_str()),
    );
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
    use bevy::prelude::*;

    use super::spawn_joining_room;
    use crate::discovery;
    use crate::identity;

    fn snapshot_in(room: &str) -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from(room),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::JoinPhase::Joining {
                    since: discovery::UnixTime::from_secs(0),
                },
                members: discovery::Members::new(),
            }),
            ..Default::default()
        }
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
            .query_filtered::<(), With<discovery::ui::JoiningRoomRoot>>();
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
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("spawn_joining_room"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![lele_bevy_preview::scene::State {
                    label: String::from("joining room"),
                    apply: preview_noop,
                }],
                timeline: None,
            },
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
