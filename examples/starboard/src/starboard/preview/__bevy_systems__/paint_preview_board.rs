use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::spectator;

use crate::starboard;

pub fn paint_preview_board(
    mut spectated: Query<&mut spectator::Spectated<starboard::canvas::Canvas>>,
    board: Option<Single<&Sprite, With<starboard::preview::PreviewBoardSprite>>>,
    mut images: ResMut<Assets<Image>>,
    mut shown_zone: Local<Option<identity::ZoneName>>,
) {
    let Some(board) = board else {
        *shown_zone = None;
        return;
    };
    let Some(mut spectated) = spectated
        .iter_mut()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Preview)
    else {
        *shown_zone = None;
        return;
    };
    let board = board.into_inner();
    let is_new_room = shown_zone.as_ref() != Some(&spectated.zone);
    if !is_new_room && spectated.state.dirty_tiles.is_empty() {
        return;
    }
    let Some(mut image) = images.get_mut(&board.image) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    if is_new_room {
        let checkered = starboard::render::CanvasView::checkered_image();
        if let Some(painted) = checkered.data
            && painted.len() == data.len()
        {
            data.copy_from_slice(&painted);
        }
        *shown_zone = Some(spectated.zone.clone());
    }
    let _ = starboard::render::repaint_dirty_tiles(&mut spectated.state, data);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator};

    use super::paint_preview_board;
    use crate::starboard;

    const CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 40, y: 40 };
    const OTHER_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 41, y: 41 };
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
                starboard::preview::bevy_systems::sync_preview_board,
                paint_preview_board,
            )
                .chain(),
        );
        app
    }

    fn start(app: &mut App, zone: &str, canvas: starboard::canvas::Canvas) -> Handle<Image> {
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                peek::PREVIEW_TARGET_WIDTH,
                peek::PREVIEW_TARGET_HEIGHT,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            ));
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from(zone),
            image: image.clone(),
        });
        app.world_mut().spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Preview,
            zone: identity::ZoneName::from(zone),
            state: canvas,
            freshness: spectator::SpectateFreshness::Live,
        });
        app.update();
        image
    }

    fn board_image(app: &mut App) -> Handle<Image> {
        let world = app.world_mut();
        let mut boards =
            world.query_filtered::<&Sprite, With<starboard::preview::PreviewBoardSprite>>();
        boards
            .single(world)
            .expect("one preview board")
            .image
            .clone()
    }

    fn board_pixel(app: &App, handle: &Handle<Image>, cell: starboard::canvas::CellPos) -> [u8; 4] {
        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(handle).expect("board image");
        let data = image.data.as_ref().expect("cpu pixels");
        let start = usize::from(cell.y)
            .saturating_mul(starboard::canvas::CANVAS_SIDE)
            .saturating_add(usize::from(cell.x))
            .saturating_mul(4);
        let mut pixel = [0_u8; 4];
        if let Some(bytes) = data.get(start..start.saturating_add(4)) {
            pixel.copy_from_slice(bytes);
        }
        pixel
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        start(&mut app, "alpha", painted(CELL, RED));

        let board = board_image(&mut app);
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::palette_rgba(RED)
        );
    }

    fn despawn_spectated(app: &mut App) {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<spectator::Spectated<starboard::canvas::Canvas>>>();
        let entities: Vec<Entity> = query.iter(app.world()).collect();
        for entity in entities {
            app.world_mut().despawn(entity);
        }
    }

    fn paint_into(
        app: &mut App,
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) {
        let world = app.world_mut();
        let mut query =
            world.query_filtered::<&mut spectator::Spectated<starboard::canvas::Canvas>, ()>();
        let mut spectated = query.single_mut(world).expect("one spectated canvas");
        assert!(starboard::canvas::write_cell(
            &mut spectated.state,
            cell,
            starboard::canvas::Layer {
                rank: 1,
                author: 1,
                color: Some(color),
                mask: starboard::canvas::FULL_MASK,
            },
        ));
    }

    #[test]
    fn a_later_stroke_repaints_without_clearing_the_earlier_one() {
        let mut app = app();
        start(&mut app, "alpha", painted(CELL, RED));

        paint_into(&mut app, OTHER_CELL, BLUE);
        app.update();

        let board = board_image(&mut app);
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::palette_rgba(RED)
        );
        assert_eq!(
            board_pixel(&app, &board, OTHER_CELL),
            starboard::render::palette_rgba(BLUE)
        );
    }

    #[test]
    fn spectating_another_room_clears_the_previous_board() {
        let mut app = app();
        start(&mut app, "alpha", painted(CELL, RED));
        despawn_spectated(&mut app);

        app.world_mut().spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Preview,
            zone: identity::ZoneName::from("beta"),
            state: starboard::canvas::Canvas::default(),
            freshness: spectator::SpectateFreshness::Live,
        });
        app.update();

        let board = board_image(&mut app);
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::checker_background_rgba_at(usize::from(CELL.x), usize::from(CELL.y))
        );
    }

    #[test]
    fn an_empty_spectated_canvas_never_touches_the_board() {
        let mut app = app();
        start(&mut app, "alpha", starboard::canvas::Canvas::default());
        app.update();
        let board = board_image(&mut app);
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::checker_background_rgba_at(usize::from(CELL.x), usize::from(CELL.y))
        );
    }
}
