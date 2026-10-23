use bevy::prelude::*;

use crate::starboard;

pub fn spawn_board(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    brush: Res<starboard::input::OwnBrush>,
) {
    let brush = brush.into_inner();
    let board = images.add(canvas_image());
    commands
        .spawn((
            starboard::render::BoardSprite,
            Sprite::from_image(board),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .with_child((
            starboard::render::BoardBorderSprite,
            Sprite::from_color(starboard::render::BOARD_BORDER_COLOR, Vec2::ONE),
            Transform::from_xyz(0.0, 0.0, -0.01),
        ));
    let fill = images.add(own_brush_image(*brush));
    let outline = images.add(starboard::render::square_image(1, vec![0_u8; 4]));
    commands
        .spawn((
            starboard::render::Cursor,
            starboard::render::Owner::Me,
            Sprite::from_image(fill),
            Visibility::Hidden,
        ))
        .with_children(|own| {
            own.spawn((
                starboard::render::CursorOutline,
                starboard::render::Owner::Me,
                Sprite::from_image(outline),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            own.spawn(starboard::render::cursor_label(
                starboard::render::OWN_LABEL_COLOR,
            ));
        });
}

// needed helper: the 1024x1024 checkered canvas kept on the CPU and re-uploaded when a tile changes
fn canvas_image() -> Image {
    let side = starboard::canvas::CANVAS_SIDE;
    let pixels = (0..side.saturating_mul(side))
        .flat_map(|index| {
            starboard::render::checker_background_rgba_at(
                index.checked_rem(side).unwrap_or(0),
                index.checked_div(side).unwrap_or(0),
            )
        })
        .collect();
    starboard::render::square_image(u32::try_from(side).unwrap_or(1024), pixels)
}

// needed helper: the solid cursor texture for the brush that is current when the board spawns
fn own_brush_image(brush: starboard::input::OwnBrush) -> Image {
    let starboard::render::BrushImage { side, pixels } = starboard::render::solid_brush_image(
        brush.shape,
        starboard::canvas::brush_radius(brush.size_index),
        brush.color,
    );
    starboard::render::square_image(side, pixels)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_board;
    use crate::starboard;

    fn preview_build(app: &mut App) {
        app.init_resource::<starboard::input::OwnBrush>()
            .init_resource::<starboard::render::CanvasView>()
            .add_systems(Startup, spawn_board)
            .add_systems(Update, starboard::render::bevy_systems::fit_board);
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
            name: String::from("starboard_spawn_board"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty board"),
                apply: preview_noop,
            }],
            timeline: None,
        }
    }

    #[test]
    fn spawn_board_ui_png_preview() {
        scene()
            .run(&config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.add_systems(Startup, spawn_board);
        app.update();

        let mut boards = app
            .world_mut()
            .query_filtered::<&Sprite, With<starboard::render::BoardSprite>>();
        let board = boards
            .single(app.world())
            .expect("one board sprite")
            .image
            .clone();
        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(&board).expect("board image");
        assert_eq!(image.texture_descriptor.size.width, 1024);
        assert_eq!(image.texture_descriptor.size.height, 1024);

        let mut cursors = app
            .world_mut()
            .query_filtered::<(&Visibility, &starboard::render::Owner), With<starboard::render::Cursor>>();
        let (visibility, owner) = cursors.single(app.world()).expect("one own cursor");
        assert_eq!(*visibility, Visibility::Hidden);
        assert!(owner.is_me());
    }
}
