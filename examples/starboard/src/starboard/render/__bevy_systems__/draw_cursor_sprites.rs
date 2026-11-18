use std::collections::BTreeMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::identity;

use crate::starboard;

pub fn draw_cursor_sprites(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    sprites: &mut starboard::render::CursorSprites,
    cursors: &BTreeMap<identity::PeerId, starboard::room::CursorUpdate>,
    brushes: &BTreeMap<identity::PeerId, starboard::identity::DailyBrush>,
    view: &starboard::render::CanvasView,
    window: Vec2,
    placement: starboard::render::CursorPlacement,
    label_kind: starboard::render::CursorLabelKind,
    layers: &RenderLayers,
) {
    for (entity, cursor_placement, owner, mut cursor_brush, mut sprite, mut transform) in
        &mut *sprites
    {
        if *cursor_placement != placement {
            continue;
        }
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
        let size_matches = images
            .get(&sprite.image)
            .is_some_and(|image| image.texture_descriptor.size.width == wanted);
        let is_brush_changed = **cursor_brush != brush;
        if (!size_matches || is_brush_changed)
            && let Some(mut image) = images.get_mut(&sprite.image)
        {
            starboard::render::replace_pixels(&mut image, brush.cursor_image(radius));
            *cursor_brush = starboard::render::CursorBrush(brush);
        }
        sprite.custom_size = Some(Vec2::splat(f32::from(outline_side) * view.scale));
        transform.translation = starboard::render::cell_center(cursor.cell, view, window);
    }
    for (peer, cursor) in cursors {
        let wanted_owner = starboard::render::Owner::Peer(peer.clone());
        let already_drawn = sprites.iter().any(|(_, cursor_placement, owner, ..)| {
            *cursor_placement == placement && *owner == wanted_owner
        });
        if already_drawn {
            continue;
        }
        let Some(brush) = brushes.get(peer).copied() else {
            continue;
        };
        let radius = starboard::canvas::brush_radius(cursor.size_index);
        let starboard::render::BrushImage { side, pixels } = brush.cursor_image(radius);
        let image = images.add(starboard::render::square_image(side, pixels));
        let mut cursor = commands.spawn((
            starboard::render::Cursor,
            placement,
            wanted_owner,
            starboard::render::CursorBrush(brush),
            Sprite::from_image(image),
            Transform::default(),
            layers.clone(),
        ));
        if label_kind == starboard::render::CursorLabelKind::MemberName {
            cursor.with_child((
                starboard::render::cursor_label(starboard::render::LABEL_COLOR, ""),
                layers.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::spectator;

    use super::draw_cursor_sprites;
    use crate::starboard;

    fn draw_test(
        mut commands: Commands,
        mut images: ResMut<Assets<Image>>,
        mut sprites: starboard::render::CursorSprites,
        cursors: Res<starboard::preview::SlotCursors>,
        brushes: Res<starboard::room::RoomBrushes>,
    ) {
        let cursors = cursors.into_inner();
        let brushes = brushes.into_inner();
        let empty = std::collections::BTreeMap::new();
        let preview = cursors
            .get(&spectator::SpectateSlot::Preview)
            .unwrap_or(&empty);
        draw_cursor_sprites(
            &mut commands,
            &mut images,
            &mut sprites,
            preview,
            brushes,
            &starboard::render::CanvasView::fit(Vec2::splat(200.0)),
            Vec2::splat(200.0),
            starboard::render::CursorPlacement::Preview,
            starboard::render::CursorLabelKind::None,
            &RenderLayers::default(),
        );
    }

    fn cursor(x: u16) -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x, y: 300 },
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.init_resource::<starboard::preview::SlotCursors>()
            .init_resource::<starboard::room::RoomBrushes>()
            .add_systems(Update, draw_test);
        app
    }

    fn preview_sprites(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut sprites = world
            .query_filtered::<&starboard::render::CursorPlacement, With<starboard::render::Cursor>>(
            );
        sprites
            .iter(world)
            .filter(|placement| **placement == starboard::render::CursorPlacement::Preview)
            .count()
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<starboard::preview::SlotCursors>()
            .init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .insert_resource(starboard::room::RoomBrushes(
                std::collections::BTreeMap::from([(
                    identity::PeerId::from("peer-a"),
                    starboard::identity::DailyBrush {
                        shape: starboard::identity::Shape::Circle,
                        color: starboard::identity::PaletteColor(0),
                    },
                )]),
            ))
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (starboard::render::bevy_systems::fit_board, draw_test).chain(),
            );
    }

    fn show_cursor(world: &mut World) {
        world
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(freenet_libp2p_bevy_plugin::lobby_room::spectator::SpectateSlot::Preview)
            .or_default()
            .insert(identity::PeerId::from("peer-a"), cursor(300));
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    #[test]
    fn draw_cursor_sprites_ui_png_preview() {
        let scene = lele_bevy_preview::scene::Scene {
            name: String::from("starboard_cursor_sprites"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("one cursor"),
                apply: show_cursor,
            }],
            timeline: None,
        };
        scene.run(&config()).expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        let peer = identity::PeerId::from("peer-a");
        app.world_mut()
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(spectator::SpectateSlot::Preview)
            .or_default()
            .insert(peer.clone(), cursor(300));
        app.insert_resource(starboard::room::RoomBrushes(
            std::collections::BTreeMap::from([(
                peer,
                starboard::identity::DailyBrush {
                    shape: starboard::identity::Shape::Circle,
                    color: starboard::identity::PaletteColor(0),
                },
            )]),
        ));
        app.update();
        assert_eq!(preview_sprites(&mut app), 1);

        app.world_mut()
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(spectator::SpectateSlot::Preview)
            .or_default()
            .clear();
        app.update();
        assert_eq!(preview_sprites(&mut app), 0);
    }
}
