use bevy::prelude::*;

use crate::ui;

pub fn spawn_roll_number(
    parent: &mut ChildSpawnerCommands,
    pill: Entity,
    slot: usize,
    value: u32,
    style: ui::roll::NumberStyle,
) -> Entity {
    let glyphs = value.to_string();
    let mut window = parent.spawn((
        ui::roll::RollNumber {
            pill,
            slot,
            value,
            row_height: style.row_height,
        },
        Node {
            width: Val::Auto,
            height: Val::Px(style.row_height),
            overflow: Overflow::clip_y(),
            flex_shrink: 0.0,
            ..default()
        },
    ));
    let window_id = window.id();
    window.with_children(|columns| {
        columns
            .spawn((
                ui::roll::RollNumberColumn,
                UiTransform::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0 * style.row_height),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
            ))
            .with_children(|column| {
                column.spawn((
                    ui::roll::RollNumberRow::Outgoing,
                    Text::new(glyphs.clone()),
                    style.text_font(),
                    style.line_height(),
                    TextColor(style.text_color),
                    TextLayout::no_wrap(),
                ));
                column.spawn((
                    ui::roll::RollNumberRow::Incoming,
                    Text::new(glyphs),
                    style.text_font(),
                    style.line_height(),
                    TextColor(style.text_color),
                    TextLayout::no_wrap(),
                ));
            });
    });
    window_id
}

#[cfg(test)]
mod tests {
    use super::spawn_roll_number;
    use crate::ui;
    use bevy::prelude::*;
    use ui::roll::NumberStyle;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    #[test]
    fn spawn_roll_number_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_roll_number"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_plugins(ui::roll::UiPlugin);
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let pill = commands
                        .spawn((
                            ChildOf(root),
                            Node {
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.44, 0.24)),
                        ))
                        .id();
                    commands.entity(pill).with_children(|parent| {
                        spawn_roll_number(&mut parent.reborrow(), pill, 0, 42, STYLE);
                    });
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("two digits"),
                apply: |_world: &mut World| {},
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
