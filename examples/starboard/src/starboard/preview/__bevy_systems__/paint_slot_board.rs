use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::spectator;

use crate::starboard;

pub fn paint_slot_board(
    mut spectated: Query<&mut spectator::Spectated<starboard::canvas::Canvas>>,
    boards: Query<(&starboard::preview::SlotBoardSprite, &Sprite)>,
    mut images: ResMut<Assets<Image>>,
    mut shown_zones: Local<BTreeMap<spectator::SpectateSlot, identity::ZoneName>>,
) {
    let live_slots: BTreeSet<spectator::SpectateSlot> =
        boards.iter().map(|(board, _)| **board).collect();
    shown_zones.retain(|slot, _| live_slots.contains(slot));
    for (board, sprite) in &boards {
        let slot = **board;
        let Some(mut spectated) = spectated
            .iter_mut()
            .find(|spectated| spectated.slot == slot)
        else {
            shown_zones.remove(&slot);
            continue;
        };
        let is_new_room = shown_zones.get(&slot) != Some(&spectated.zone);
        if !is_new_room && spectated.state.dirty_tiles.is_empty() {
            continue;
        }
        let Some(mut image) = images.get_mut(&sprite.image) else {
            continue;
        };
        let Some(data) = image.data.as_mut() else {
            continue;
        };
        if is_new_room {
            let checkered = starboard::render::CanvasView::checkered_image();
            if let Some(painted) = checkered.data
                && painted.len() == data.len()
            {
                data.copy_from_slice(&painted);
            }
            shown_zones.insert(slot, spectated.zone.clone());
        }
        let _ = starboard::render::repaint_dirty_tiles(&mut spectated.state, data);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator};

    use super::paint_slot_board;
    use crate::starboard;

    const PREVIEW_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 40, y: 40 };
    const STREAM_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 300, y: 200 };
    const RED: starboard::identity::PaletteColor = starboard::identity::PaletteColor(0);
    const BLUE: starboard::identity::PaletteColor = starboard::identity::PaletteColor(6);

    fn painted(
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) -> starboard::canvas::Canvas {
        let mut canvas = starboard::canvas::Canvas::default();
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            cell,
            starboard::canvas::Layer {
                rank: 1,
                author: 1,
                color: Some(color),
                mask: starboard::canvas::FULL_MASK,
            },
        ));
        canvas
    }

    fn target_image(app: &mut App, width: u32, height: u32) -> Handle<Image> {
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                width,
                height,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            ))
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.add_systems(
            Update,
            (
                starboard::preview::bevy_systems::sync_slot_boards,
                paint_slot_board,
            )
                .chain(),
        );
        app
    }

    fn start(
        app: &mut App,
        slot: spectator::SpectateSlot,
        zone: &str,
        canvas: starboard::canvas::Canvas,
    ) {
        let (width, height) = match slot {
            spectator::SpectateSlot::Preview => {
                (peek::PREVIEW_TARGET_WIDTH, peek::PREVIEW_TARGET_HEIGHT)
            }
            spectator::SpectateSlot::Stream => {
                (peek::STREAM_TARGET_WIDTH, peek::STREAM_TARGET_HEIGHT)
            }
        };
        let image = target_image(app, width, height);
        match slot {
            spectator::SpectateSlot::Preview => {
                app.insert_resource(peek::RoomPreviewTarget {
                    zone: identity::ZoneName::from(zone),
                    image,
                });
            }
            spectator::SpectateSlot::Stream => {
                app.insert_resource(peek::RoomStreamTarget {
                    zone: identity::ZoneName::from(zone),
                    image,
                });
            }
        }
        app.world_mut().spawn(spectator::Spectated {
            slot,
            zone: identity::ZoneName::from(zone),
            state: canvas,
            freshness: spectator::SpectateFreshness::Live,
        });
        app.update();
    }

    fn board_image(app: &mut App, slot: spectator::SpectateSlot) -> Handle<Image> {
        let world = app.world_mut();
        let mut boards = world.query::<(&starboard::preview::SlotBoardSprite, &Sprite)>();
        boards
            .iter(world)
            .find(|(board, _)| ***board == slot)
            .map(|(_, sprite)| sprite.image.clone())
            .expect("a board")
    }

    fn board_pixel(app: &App, handle: &Handle<Image>, cell: starboard::canvas::CellPos) -> [u8; 4] {
        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(handle).expect("board image");
        let data = image.data.as_ref().expect("cpu pixels");
        let start = usize::from(cell.y)
            .saturating_mul(starboard::canvas::CANVAS_SIDE)
            .saturating_add(usize::from(cell.x))
            .saturating_mul(4);
        data.get(start..start.saturating_add(4))
            .map(|pixel: &[u8]| {
                let mut out = [0_u8; 4];
                out.copy_from_slice(pixel);
                out
            })
            .expect("board pixel")
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        start(
            &mut app,
            spectator::SpectateSlot::Preview,
            "alpha",
            painted(PREVIEW_CELL, RED),
        );

        let board = board_image(&mut app, spectator::SpectateSlot::Preview);
        assert_eq!(
            board_pixel(&app, &board, PREVIEW_CELL),
            starboard::render::palette_rgba(RED)
        );
    }

    #[test]
    fn each_slot_paints_its_own_board_from_its_own_canvas() {
        let mut app = app();
        start(
            &mut app,
            spectator::SpectateSlot::Preview,
            "alpha",
            painted(PREVIEW_CELL, RED),
        );
        start(
            &mut app,
            spectator::SpectateSlot::Stream,
            "beta",
            painted(STREAM_CELL, BLUE),
        );

        let preview_board = board_image(&mut app, spectator::SpectateSlot::Preview);
        assert_eq!(
            board_pixel(&app, &preview_board, PREVIEW_CELL),
            starboard::render::palette_rgba(RED)
        );
        assert_eq!(
            board_pixel(&app, &preview_board, STREAM_CELL),
            starboard::render::checker_background_rgba_at(
                usize::from(STREAM_CELL.x),
                usize::from(STREAM_CELL.y)
            )
        );

        let stream_board = board_image(&mut app, spectator::SpectateSlot::Stream);
        assert_eq!(
            board_pixel(&app, &stream_board, STREAM_CELL),
            starboard::render::palette_rgba(BLUE)
        );
        assert_eq!(
            board_pixel(&app, &stream_board, PREVIEW_CELL),
            starboard::render::checker_background_rgba_at(
                usize::from(PREVIEW_CELL.x),
                usize::from(PREVIEW_CELL.y)
            )
        );
    }
}
