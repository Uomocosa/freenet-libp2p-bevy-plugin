use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{LeaveDoorRoot, LeavePromptOpen, UiRoot};

pub fn sync_in_room(
    snapshot: Res<discovery::Snapshot>,
    mut lobbies: Query<&mut Node, (With<UiRoot>, Without<LeaveDoorRoot>)>,
    mut doors: Query<&mut Node, (With<LeaveDoorRoot>, Without<UiRoot>)>,
    mut prompt: ResMut<LeavePromptOpen>,
) {
    if !snapshot.is_changed() {
        return;
    }
    let snapshot = snapshot.into_inner();
    let in_room = snapshot.room.is_some();
    for mut node in &mut lobbies {
        node.display = display_for(!in_room);
    }
    for mut node in &mut doors {
        node.display = display_for(in_room);
    }
    if !in_room && **prompt {
        **prompt = false;
    }
}

const fn display_for(shown: bool) -> Display {
    if shown { Display::Flex } else { Display::None }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_in_room;
    use crate::{discovery, identity};

    fn in_room() -> discovery::Snapshot {
        discovery::Snapshot {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                members: discovery::Members::new(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(in_room());
        app.init_resource::<discovery::ui::LeavePromptOpen>();
        let lobby = app
            .world_mut()
            .spawn((discovery::ui::UiRoot, Node::default()))
            .id();
        let door = app
            .world_mut()
            .spawn((discovery::ui::LeaveDoorRoot, Node::default()))
            .id();
        app.add_systems(Update, sync_in_room);
        app.update();
        let display = |app: &App, entity| app.world().get::<Node>(entity).map(|node| node.display);
        assert_eq!(display(&app, lobby), Some(Display::None));
        assert_eq!(display(&app, door), Some(Display::Flex));
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<discovery::Snapshot>();
        app.init_resource::<discovery::ui::LeavePromptOpen>();
        app.add_systems(
            Startup,
            (
                discovery::ui::bevy_systems::spawn_root,
                discovery::ui::bevy_systems::spawn_leave_door,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, sync_in_room);
    }

    const fn preview_noop(_world: &mut World) {}

    fn enter_room(world: &mut World) {
        world.insert_resource(in_room());
    }

    #[test]
    fn sync_in_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("sync_in_room"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![
                    lele_bevy_preview::scene::State {
                        label: String::from("lobby"),
                        apply: preview_noop,
                    },
                    lele_bevy_preview::scene::State {
                        label: String::from("in room"),
                        apply: enter_room,
                    },
                ],
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
