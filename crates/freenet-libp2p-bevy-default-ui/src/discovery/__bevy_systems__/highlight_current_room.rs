use bevy::prelude::*;

use crate::discovery;
use core_discovery::state_machine::DiscoveryState;
use discovery::basic::constants::{CURRENT_ROOM_COLOR, ROOM_COLOR};
use discovery::{IsCurrentRoom, RoomButton};
use freenet_libp2p_bevy_core::discovery as core_discovery;

pub fn highlight_current_room(
    mut commands: Commands,
    state: Option<Res<DiscoveryState>>,
    mut rows: Query<(
        Entity,
        &RoomButton,
        &mut BackgroundColor,
        Option<&IsCurrentRoom>,
    )>,
) {
    let current = state.and_then(|state| {
        state
            .into_inner()
            .room
            .as_ref()
            .map(|room| room.name.clone())
    });
    for (entity, button, mut background, marker) in &mut rows {
        let is_current = current.as_ref().is_some_and(|room| *room == **button);
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
    use crate::discovery;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::ui_widgets::Button;

    use super::highlight_current_room;
    use core_discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_core::discovery as core_discovery;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::testing;
    use freenet_libp2p_bevy_core::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let room = identity::RoomName::from("alpha");
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(core_discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: core_discovery::JoinPhase::Ready,
                members: core_discovery::Members::new(),
                how_entered: core_discovery::HowRoomWasEntered::Created,
            })
            .build();
        app.insert_resource(state);
        let entity = app
            .world_mut()
            .spawn((discovery::RoomButton(room), BackgroundColor(Color::BLACK)))
            .id();
        app.add_systems(Update, highlight_current_room);
        app.update();
        assert!(
            app.world()
                .entity(entity)
                .contains::<discovery::IsCurrentRoom>()
        );
    }

    fn preview_snapshot(room: &str) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(core_discovery::JoinedRoom {
                name: identity::RoomName::from(room),
                created_at: time::UnixTime::EPOCH,
                phase: core_discovery::JoinPhase::Ready,
                members: core_discovery::Members::new(),
                how_entered: core_discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn spawn_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        for name in ["alpha", "beta"] {
            commands.spawn((
                ChildOf(root),
                discovery::RoomButton(identity::RoomName::from(name)),
                Button,
                Hovered::default(),
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
        world.resource_mut::<DiscoveryState>().room = Some(core_discovery::JoinedRoom {
            name: identity::RoomName::from("beta"),
            created_at: time::UnixTime::EPOCH,
            phase: core_discovery::JoinPhase::Ready,
            members: core_discovery::Members::new(),
            how_entered: core_discovery::HowRoomWasEntered::Created,
        });
    }

    #[test]
    fn highlight_current_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
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
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
