//! Status overlay: a top bar with the current room once joined.
//! Reads `DiscoveryState` only; the connecting spinner is the plugin's own screen.
//! The peer list lives in the default-ui crate's roster bubble.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::discovery;

use examples_shared::status;

const BAR_COLOR: Color = Color::srgb(0.106, 0.122, 0.153);

pub struct Overlay;

impl Plugin for Overlay {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (sync_visibility, sync_room_bar));
    }
}

#[derive(Component)]
pub struct RoomBar;

#[derive(Component)]
pub struct RoomView;

#[derive(Component)]
pub struct RoomTitle;

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        RoomBar,
        RoomView,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(40.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            ..default()
        },
        BackgroundColor(BAR_COLOR),
        children![(RoomTitle, Text::new(""), TextLayout::no_wrap())],
    ));
}

fn sync_visibility(
    state: Option<Res<DiscoveryState>>,
    mut views: Query<&mut Node, With<RoomView>>,
) {
    let Some(state) = state.as_ref() else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    for mut view in &mut views {
        view.display = display_for(
            state
                .room
                .as_ref()
                .is_some_and(|room| room.phase == discovery::JoinPhase::Ready),
        );
    }
}

fn sync_room_bar(
    state: Option<Res<DiscoveryState>>,
    local_user: Res<status::LocalUser>,
    mut title: Single<&mut Text, With<RoomTitle>>,
) {
    let Some(state) = state.as_ref() else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let Some(room) = state.room.as_ref() else {
        return;
    };
    **title = Text::new(format!(
        "Current room: {}   |   you: {}",
        room.name.as_str(),
        local_user.as_str()
    ));
}

const fn display_for(is_shown: bool) -> Display {
    if is_shown {
        Display::Flex
    } else {
        Display::None
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;

    use freenet_libp2p_bevy_plugin::{discovery, identity, time};

    use super::{Overlay, RoomBar, display_for};

    fn preview_build(app: &mut App) {
        app.insert_resource(
            freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
                identity::PeerId::from("me"),
            )
            .build(),
        );
        app.insert_resource(examples_shared::status::LocalUser(
            identity::Username::from("alice"),
        ));
        app.add_plugins(Overlay);
        join_first_room(app.world_mut());
    }

    fn join_first_room(world: &mut World) {
        world.resource_mut::<DiscoveryState>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
    }

    const fn preview_noop(_world: &mut World) {}

    fn rename_room(world: &mut World) {
        world.resource_mut::<DiscoveryState>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("gamma"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("overlay"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("room joined"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("another room"),
                    apply: rename_room,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn test_usage() {
        assert!(matches!(display_for(true), Display::Flex));
        assert!(matches!(display_for(false), Display::None));
        let mut app = App::new();
        app.insert_resource(
            freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
                identity::PeerId::from("me"),
            )
            .build(),
        );
        app.insert_resource(examples_shared::status::LocalUser(
            identity::Username::from("alice"),
        ));
        app.add_plugins(Overlay);
        app.update();
        let mut bars = app.world_mut().query_filtered::<(), With<RoomBar>>();
        assert_eq!(bars.iter(app.world()).count(), 1);
    }

    #[test]
    fn overlay_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }

    #[test]
    fn overlay_ui_scene_preview() {
        scene().run(&config()).expect("preview");
    }
}
