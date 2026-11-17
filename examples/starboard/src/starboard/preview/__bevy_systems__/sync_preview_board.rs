use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::peek;

use crate::starboard;

pub fn sync_preview_board(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    target: Option<Res<peek::RoomPreviewTarget>>,
    boards: Query<Entity, With<starboard::preview::PreviewBoardSprite>>,
) {
    let Some(_target) = target else {
        for board in &boards {
            commands.entity(board).despawn();
        }
        return;
    };
    if !boards.is_empty() {
        return;
    }
    let image = images.add(starboard::render::CanvasView::checkered_image());
    commands.spawn((
        starboard::preview::PreviewBoardSprite,
        Sprite {
            image,
            custom_size: Some(Vec2::splat(starboard::preview::PREVIEW_BOARD_SIDE)),
            ..default()
        },
        RenderLayers::layer(starboard::preview::PREVIEW_LAYER),
    ));
}

#[cfg(test)]
mod tests {
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator};

    use super::sync_preview_board;
    use crate::starboard;

    #[derive(Component)]
    struct ViewportSlot;

    const VIEWPORT_WIDTH: f32 = 320.0;
    const VIEWPORT_HEIGHT: f32 = 200.0;

    fn target_image(app: &mut App) -> Handle<Image> {
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                peek::PREVIEW_TARGET_WIDTH,
                peek::PREVIEW_TARGET_HEIGHT,
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
                sync_preview_board,
                starboard::preview::bevy_systems::paint_preview_board,
            )
                .chain(),
        );
        app
    }

    fn spectate(app: &mut App, zone: &str, canvas: starboard::canvas::Canvas) {
        app.insert_resource(spectator::Spectated {
            zone: identity::ZoneName::from(zone),
            state: canvas,
            freshness: spectator::SpectateFreshness::Live,
        });
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

    fn board_layers(app: &mut App) -> RenderLayers {
        let world = app.world_mut();
        let mut boards =
            world.query_filtered::<&RenderLayers, With<starboard::preview::PreviewBoardSprite>>();
        boards.single(world).expect("one preview board").clone()
    }

    fn board_pixel(app: &App, handle: &Handle<Image>, cell: starboard::canvas::CellPos) -> [u8; 4] {
        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(handle).expect("board image");
        let data = image.data.as_ref().expect("cpu pixels");
        let side = starboard::canvas::CANVAS_SIDE;
        let start = usize::from(cell.y)
            .saturating_mul(side)
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

    fn paint(
        app: &mut App,
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) {
        let mut spectated = app
            .world_mut()
            .resource_mut::<spectator::Spectated<starboard::canvas::Canvas>>();
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

    const CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 40, y: 40 };
    const OTHER_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 41, y: 41 };
    const RED: starboard::identity::PaletteColor = starboard::identity::PaletteColor(0);
    const BLUE: starboard::identity::PaletteColor = starboard::identity::PaletteColor(6);

    #[test]
    fn test_usage() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        let mut canvas = starboard::canvas::Canvas::default();
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            CELL,
            starboard::canvas::Layer {
                rank: 1,
                author: 1,
                color: Some(RED),
                mask: starboard::canvas::FULL_MASK,
            },
        ));
        spectate(&mut app, "alpha", canvas);
        app.update();

        let board = board_image(&mut app);
        assert_eq!(
            board_layers(&mut app),
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::palette_rgba(RED)
        );
    }

    #[test]
    fn dropping_the_target_despawns_the_preview_board() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        spectate(&mut app, "alpha", starboard::canvas::Canvas::default());
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<Entity, With<starboard::preview::PreviewBoardSprite>>()
                .single(app.world())
                .is_ok()
        );

        app.world_mut().remove_resource::<peek::RoomPreviewTarget>();
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<Entity, With<starboard::preview::PreviewBoardSprite>>()
                .single(app.world())
                .is_err()
        );
    }

    #[test]
    fn a_later_stroke_repaints_without_clearing_the_earlier_one() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        spectate(&mut app, "alpha", starboard::canvas::Canvas::default());
        app.update();
        paint(&mut app, CELL, RED);
        app.update();
        paint(&mut app, OTHER_CELL, BLUE);
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
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        let mut canvas = starboard::canvas::Canvas::default();
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            CELL,
            starboard::canvas::Layer {
                rank: 1,
                author: 1,
                color: Some(RED),
                mask: starboard::canvas::FULL_MASK,
            },
        ));
        spectate(&mut app, "alpha", canvas);
        app.update();

        spectate(&mut app, "beta", starboard::canvas::Canvas::default());
        app.update();

        let board = board_image(&mut app);
        assert_eq!(
            board_pixel(&app, &board, CELL),
            starboard::render::checker_background_rgba_at(usize::from(CELL.x), usize::from(CELL.y))
        );
    }

    fn paint_block(
        canvas: &mut starboard::canvas::Canvas,
        origin: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) {
        for dy in 0..8_u16 {
            for dx in 0..8_u16 {
                let _ = starboard::canvas::write_cell(
                    canvas,
                    starboard::canvas::CellPos {
                        x: origin.x.saturating_add(dx),
                        y: origin.y.saturating_add(dy),
                    },
                    starboard::canvas::Layer {
                        rank: 1,
                        author: 1,
                        color: Some(color),
                        mask: starboard::canvas::FULL_MASK,
                    },
                );
            }
        }
    }

    fn painted_canvas() -> starboard::canvas::Canvas {
        let mut canvas = starboard::canvas::Canvas::default();
        for step in 0..=40_u16 {
            let offset = step.saturating_mul(20);
            paint_block(
                &mut canvas,
                starboard::canvas::CellPos {
                    x: 100_u16.saturating_add(offset),
                    y: 100_u16.saturating_add(offset),
                },
                RED,
            );
            paint_block(
                &mut canvas,
                starboard::canvas::CellPos {
                    x: 900_u16.saturating_sub(offset),
                    y: 100_u16.saturating_add(offset),
                },
                BLUE,
            );
        }
        canvas
    }

    fn spawn_viewport(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            ViewportSlot,
            Node {
                width: Val::Px(VIEWPORT_WIDTH),
                height: Val::Px(VIEWPORT_HEIGHT),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_viewport);
        app.add_systems(
            Update,
            (
                starboard::preview::bevy_systems::sync_preview_camera,
                sync_preview_board,
                starboard::preview::bevy_systems::paint_preview_board,
            )
                .chain(),
        );
    }

    fn show_spectated_board(world: &mut World) {
        let image = world
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                peek::PREVIEW_TARGET_WIDTH,
                peek::PREVIEW_TARGET_HEIGHT,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            ));
        let zone = identity::ZoneName::from("alpha");
        world.insert_resource(peek::RoomPreviewTarget {
            zone: zone.clone(),
            image: image.clone(),
        });
        world.insert_resource(spectator::Spectated {
            zone,
            state: painted_canvas(),
            freshness: spectator::SpectateFreshness::Live,
        });
        let mut slots = world.query_filtered::<Entity, With<ViewportSlot>>();
        if let Some(slot) = slots.iter(world).next() {
            world.entity_mut(slot).insert(ImageNode::new(image));
        }
    }

    const fn preview_noop(_world: &mut World) {}

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_room_preview"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("nothing hovered"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("hovered room board"),
                    apply: show_spectated_board,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn sync_preview_board_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }
}
