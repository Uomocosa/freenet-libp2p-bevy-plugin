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
    mut sprites: Query<
        (
            Entity,
            &starboard::render::Owner,
            &mut Sprite,
            &mut Transform,
        ),
        With<starboard::render::Cursor>,
    >,
) {
    let view = view.into_inner();
    let cursors = cursors.into_inner();
    let brushes = brushes.into_inner();
    let Ok(window) = windows.single() else {
        return;
    };
    let window_size = window.size();
    for (entity, owner, mut sprite, mut transform) in &mut sprites {
        let starboard::render::Owner::Peer(peer) = owner else {
            continue;
        };
        let Some(cursor) = cursors.get(peer) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Some(brush) = brushes.get(peer).copied() else {
            continue;
        };
        let radius = starboard::canvas::brush_radius(cursor.size_index);
        let outline_side = radius.saturating_mul(2).saturating_add(1);
        let wanted = u32::from(outline_side);
        let needs_texture = images
            .get(&sprite.image)
            .is_none_or(|image| image.texture_descriptor.size.width != wanted);
        if needs_texture && let Some(mut image) = images.get_mut(&sprite.image) {
            starboard::render::replace_pixels(&mut image, remote_brush_image(brush, radius));
        }
        sprite.custom_size = Some(Vec2::splat(f32::from(outline_side) * view.scale));
        transform.translation = starboard::render::cell_center(cursor.cell, view, window_size);
    }
    for (peer, cursor) in cursors.iter() {
        let wanted_owner = starboard::render::Owner::Peer(peer.clone());
        if sprites
            .iter()
            .any(|(_, owner, _, _)| *owner == wanted_owner)
        {
            continue;
        }
        let Some(brush) = brushes.get(peer).copied() else {
            continue;
        };
        let radius = starboard::canvas::brush_radius(cursor.size_index);
        let starboard::render::BrushImage { side, pixels } = remote_brush_image(brush, radius);
        let image = images.add(starboard::render::square_image(side, pixels));
        commands
            .spawn((
                starboard::render::Cursor,
                wanted_owner,
                Sprite::from_image(image),
                Transform::default(),
            ))
            .with_child(starboard::render::cursor_label(
                starboard::render::LABEL_COLOR,
            ));
    }
}

// needed helper: a peer's cursor is a half-transparent fill inside a solid edge of its colour
fn remote_brush_image(
    brush: starboard::identity::DailyBrush,
    radius: u16,
) -> starboard::render::BrushImage {
    let edge = starboard::render::palette_rgba(brush.color);
    let [red, green, blue, _] = edge;
    let fill = [red, green, blue, starboard::render::PREVIEW_FILL_ALPHA];
    starboard::render::brush_image(brush.shape, radius, fill, edge)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::draw_remote_cursors;
    use crate::starboard;

    fn members(peers: &[&str]) -> discovery::Members {
        let mut members = discovery::Members::new();
        for peer in peers {
            members.insert(
                identity::PeerId::from(*peer),
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: None,
                        presence: discovery::Presence::InRoom,
                    },
                    status: discovery::LinkStatus::Connected,
                },
            );
        }
        members
    }

    fn room_brushes(members: &discovery::Members) -> starboard::room::RoomBrushes {
        let mut brushes = starboard::room::RoomBrushes::default();
        brushes.0 = starboard::identity::room_brushes(
            members.keys(),
            discovery::UnixTime::from_secs(0),
            discovery::UnixTime::from_secs(1),
        );
        brushes
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
            [(identity::PeerId::from("peer-a"), cursor(300))]
                .into_iter()
                .collect(),
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
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
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
