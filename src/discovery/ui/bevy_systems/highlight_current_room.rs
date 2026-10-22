use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{CURRENT_ROOM_COLOR, ROOM_COLOR};
use discovery::ui::{IsCurrentRoom, RoomButton};

pub fn highlight_current_room(
    mut commands: Commands,
    status: Res<discovery::Status>,
    mut rows: Query<(
        Entity,
        &RoomButton,
        &mut BackgroundColor,
        Option<&IsCurrentRoom>,
    )>,
) {
    let status = status.into_inner();
    let current = status.room.as_ref().map(|room| &room.name);
    for (entity, button, mut background, marker) in &mut rows {
        let is_current = current.is_some_and(|room| *room == **button);
        let color = if is_current {
            CURRENT_ROOM_COLOR
        } else {
            ROOM_COLOR
        };
        let BackgroundColor(existing) = &mut *background;
        if *existing != color {
            *existing = color;
        }
        match (is_current, marker.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(IsCurrentRoom);
            }
            (false, true) => {
                commands.entity(entity).remove::<IsCurrentRoom>();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::discovery;
    use crate::identity;

    use super::highlight_current_room;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let room = identity::RoomName::from("alpha");
        let status = discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..Default::default()
        };
        app.insert_resource(status);
        let entity = app
            .world_mut()
            .spawn((
                discovery::ui::RoomButton(room),
                BackgroundColor(Color::BLACK),
            ))
            .id();
        app.add_systems(Update, highlight_current_room);
        app.update();
        assert!(
            app.world()
                .entity(entity)
                .contains::<discovery::ui::IsCurrentRoom>()
        );
    }

    fn preview_snapshot(room: &str) -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from(room),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..Default::default()
        }
    }

    fn spawn_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::centered_root())
            .id();
        for name in ["alpha", "beta"] {
            commands.spawn((
                ChildOf(root),
                discovery::ui::RoomButton(identity::RoomName::from(name)),
                Button,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(28.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                children![Text::new(name)],
            ));
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_snapshot("alpha"));
        app.add_systems(Startup, spawn_rows);
        app.add_systems(Update, highlight_current_room);
    }

    const fn preview_noop(_world: &mut World) {}

    fn pick_beta(world: &mut World) {
        world.resource_mut::<discovery::Status>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("beta"),
            created_at: discovery::UnixTime::from(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
        });
    }

    #[test]
    fn highlight_current_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("highlight_current_room"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![
                    lele_bevy_preview::scene::State {
                        label: String::from("alpha current"),
                        apply: preview_noop,
                    },
                    lele_bevy_preview::scene::State {
                        label: String::from("beta current"),
                        apply: pick_beta,
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
