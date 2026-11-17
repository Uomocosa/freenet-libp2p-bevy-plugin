use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::starboard;

pub fn draw_remote_cursors(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    view: Res<starboard::render::CanvasView>,
    cursors: Res<starboard::room::RemoteCursors>,
    brushes: Res<starboard::room::RoomBrushes>,
    mut sprites: starboard::render::CursorSprites,
) {
    let view = view.into_inner();
    let cursors = cursors.into_inner();
    let brushes = brushes.into_inner();
    let Ok(window) = windows.single() else {
        return;
    };
    starboard::render::bevy_systems::draw_cursor_sprites(
        &mut commands,
        &mut images,
        &mut sprites,
        cursors,
        brushes,
        view,
        window.size(),
        starboard::render::CursorPlacement::InRoom,
        starboard::render::CursorLabelKind::MemberName,
        &RenderLayers::default(),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;
    use freenet_libp2p_bevy_plugin::time;

    use super::draw_remote_cursors;
    use crate::starboard;

    fn members(peers: &[&str]) -> discovery::Members {
        let mut members = discovery::Members::new();
        for peer in peers {
            members.insert(
                identity::PeerId::from(*peer),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(*peer),
                        addrs: Vec::new(),
                    },
                    updated_at: time::UnixTime::EPOCH,
                    display_name: None,
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        members
    }

    fn room_brushes(members: &discovery::Members) -> starboard::room::RoomBrushes {
        starboard::room::RoomBrushes(starboard::identity::room_brushes(
            members.keys(),
            time::UnixTime::EPOCH,
            time::UnixTime::from_secs(1),
        ))
    }

    fn cursor(x: u16) -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x, y: 300 },
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    fn preview_build(app: &mut App) {
        let members = members(&["peer-a", "peer-b"]);
        app.insert_resource(room_brushes(&members));
        app.insert_resource(starboard::room::RemoteCursors(
            std::iter::once((identity::PeerId::from("peer-a"), cursor(300))).collect(),
        ));
        app.init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    starboard::render::bevy_systems::fit_board,
                    draw_remote_cursors,
                ),
            );
    }

    const fn preview_noop(_world: &mut World) {}

    fn two_cursors(world: &mut World) {
        world
            .resource_mut::<starboard::room::RemoteCursors>()
            .insert(identity::PeerId::from("peer-b"), cursor(700));
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
            name: String::from("starboard_remote_cursors"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one peer"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("two peers"),
                    apply: two_cursors,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn draw_remote_cursors_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }

    #[test]
    fn test_usage_1() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        let members = members(&["peer-a"]);
        let peer_a = identity::PeerId::from("peer-a");
        app.insert_resource(room_brushes(&members));
        app.init_resource::<starboard::room::RemoteCursors>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.world_mut().spawn((
            bevy::window::PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
        app.add_systems(Update, draw_remote_cursors);

        app.world_mut()
            .resource_mut::<starboard::room::RemoteCursors>()
            .insert(peer_a.clone(), cursor(300));
        app.update();

        let changed = {
            let brushes = app.world().resource::<starboard::room::RoomBrushes>();
            let old = brushes
                .get(&peer_a)
                .copied()
                .expect("peer-a brush assigned");
            starboard::identity::DailyBrush {
                shape: if old.shape == starboard::identity::Shape::Circle {
                    starboard::identity::Shape::Square
                } else {
                    starboard::identity::Shape::Circle
                },
                color: if old.color == starboard::identity::PaletteColor(0) {
                    starboard::identity::PaletteColor(1)
                } else {
                    starboard::identity::PaletteColor(0)
                },
            }
        };
        app.world_mut()
            .resource_mut::<starboard::room::RoomBrushes>()
            .0
            .insert(peer_a, changed);
        app.update();

        let mut sprites = app
            .world_mut()
            .query_filtered::<&Sprite, With<starboard::render::Cursor>>();
        let handle = sprites
            .single(app.world())
            .expect("one remote cursor")
            .image
            .clone();
        let image = app
            .world()
            .resource::<Assets<Image>>()
            .get(&handle)
            .expect("cursor image");
        let expected =
            changed.cursor_image(starboard::canvas::brush_radius(cursor(300).size_index));
        assert_eq!(image.texture_descriptor.size.width, expected.side);
        assert_eq!(image.data.as_deref(), Some(expected.pixels.as_slice()));
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        let members = members(&["peer-a"]);
        app.insert_resource(room_brushes(&members));
        app.init_resource::<starboard::room::RemoteCursors>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.world_mut().spawn((
            bevy::window::PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
        app.add_systems(Update, draw_remote_cursors);

        app.update();
        let mut sprites = app
            .world_mut()
            .query_filtered::<(), With<starboard::render::Cursor>>();
        assert_eq!(sprites.iter(app.world()).count(), 0);

        app.world_mut()
            .resource_mut::<starboard::room::RemoteCursors>()
            .insert(identity::PeerId::from("peer-a"), cursor(300));
        app.update();
        let mut sprites = app
            .world_mut()
            .query_filtered::<(), With<starboard::render::Cursor>>();
        assert_eq!(sprites.iter(app.world()).count(), 1);

        app.world_mut()
            .resource_mut::<starboard::room::RemoteCursors>()
            .clear();
        app.update();
        let mut sprites = app
            .world_mut()
            .query_filtered::<(), With<starboard::render::Cursor>>();
        assert_eq!(sprites.iter(app.world()).count(), 0);
    }
}
