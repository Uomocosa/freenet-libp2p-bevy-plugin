use crate::ui;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

use ui::discovery::{ROOM_NAME_SCROLL_PER_SEC, RoomNameClip, RoomNameScroller};

pub fn scroll_room_names(
    time: Res<Time>,
    mut scrollers: Query<(&RoomNameScroller, &ChildOf, &Children, &mut UiTransform)>,
    clips: Query<&ComputedNode, With<RoomNameClip>>,
    nodes: Query<&ComputedNode>,
) {
    let elapsed_secs = time.into_inner().elapsed_secs();
    for (_scroller, parent, children, mut transform) in &mut scrollers {
        let Some(clip_width) = clips
            .get(parent.get())
            .ok()
            .filter(|clip| clip.size.x > 0.0)
            .map(|clip| clip.size.x)
        else {
            continue;
        };
        let text_width = children
            .iter()
            .filter_map(|child| nodes.get(child).ok())
            .filter(|text| text.size.x > 0.0)
            .map(|text| text.size.x)
            .next()
            .unwrap_or(0.0);
        let wanted = if text_width > clip_width {
            let travel = text_width + clip_width;
            Val2::px(
                clip_width - (ROOM_NAME_SCROLL_PER_SEC * elapsed_secs).rem_euclid(travel),
                0.0,
            )
        } else {
            Val2::ZERO
        };
        if transform.translation != wanted {
            transform.translation = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use crate::ui::basic;
    use bevy::prelude::*;

    use super::scroll_room_names;

    #[test]
    fn scroll_room_names_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("scroll_room_names"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(basic::constants::ROOM_COLOR),
                        children![(
                            ui::discovery::RoomNameClip,
                            Node {
                                width: Val::Px(ui::discovery::ROOM_NAME_WIDTH),
                                overflow: Overflow::clip_x(),
                                ..default()
                            },
                            children![(
                                ui::discovery::RoomNameScroller,
                                UiTransform::default(),
                                Node::default(),
                                children![(
                                    Text::new("very-long-room-name-that-never-fits".to_string()),
                                    TextLayout::no_wrap(),
                                )],
                            )],
                        )],
                    ));
                });
                app.add_systems(Update, scroll_room_names);
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("clipped name"),
                apply: |_world: &mut World| {},
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("scroll"),
                frames: 60,
                fps: 30,
                apply: |_world: &mut World| {},
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
