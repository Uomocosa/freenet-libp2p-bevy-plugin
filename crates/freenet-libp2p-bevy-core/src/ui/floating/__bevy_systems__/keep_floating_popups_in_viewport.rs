use bevy::math::{Rect, Vec2};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::ui;

pub fn keep_floating_popups_in_viewport(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut popups: Query<(
        &ui::floating::FloatingPopup,
        &ComputedNode,
        Option<&ChildOf>,
        &mut Node,
    )>,
) {
    let Some(window) = window else {
        return;
    };
    for (popup, popup_node, parent, mut node) in &mut popups {
        let inset = *popup.edge_margin;
        let area = Rect::from_corners(
            Vec2::new(inset, inset),
            Vec2::new(window.width() - inset, window.height() - inset),
        );
        let inverse = popup_node.inverse_scale_factor();
        let popup_size = popup_node.size();
        let size = Vec2::new(popup_size.x * inverse, popup_size.y * inverse);
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }
        let anchor = match popup.anchor {
            ui::floating::PopupAnchor::Parent => parent.map(ChildOf::parent),
            ui::floating::PopupAnchor::Entity(entity) => Some(entity),
        };
        let Some(anchor) = anchor else {
            continue;
        };
        let Ok((anchor_node, anchor_transform)) = nodes.get(anchor) else {
            continue;
        };
        let placement = ui::floating::PopupPlacement {
            anchor: ui::floating::PopupPlacement::rect_of(anchor_node, anchor_transform),
            popup: size,
            area,
            preferred_side: popup.preferred_side,
            cross_align: popup.cross_align,
            gap: popup.gap,
        };
        let mut wanted = placement.top_left();
        if let Some(parent) = parent
            && let Ok((parent_node, parent_transform)) = nodes.get(parent.parent())
        {
            let parent_rect = ui::floating::PopupPlacement::rect_of(parent_node, parent_transform);
            wanted = Vec2::new(wanted.x - parent_rect.min.x, wanted.y - parent_rect.min.y);
        }
        if node.left != Val::Px(wanted.x) {
            node.left = Val::Px(wanted.x);
        }
        if node.top != Val::Px(wanted.y) {
            node.top = Val::Px(wanted.y);
        }
        if node.right != Val::Auto {
            node.right = Val::Auto;
        }
        if node.bottom != Val::Auto {
            node.bottom = Val::Auto;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::keep_floating_popups_in_viewport;
    use crate::ui;

    #[test]
    fn keep_floating_popups_in_viewport_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("keep_floating_popups_in_viewport"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn((
                            lele_bevy_preview::scene::Scene::centered_root(),
                            BackgroundColor(Color::srgb(0.85, 0.86, 0.9)),
                        ))
                        .id();
                    let anchor = commands
                        .spawn((
                            ChildOf(root),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(840.0),
                                top: Val::Px(120.0),
                                width: Val::Px(120.0),
                                height: Val::Px(36.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
                        ))
                        .id();
                    commands.spawn((
                        Name::new("floating-popup"),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(18.0),
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
                        ChildOf(anchor),
                        children![Text::new("an occupancy tooltip wider than its pill")],
                    ));
                });
                app.add_systems(Update, keep_floating_popups_in_viewport);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("popup overflowing the right edge"),
                    apply: |world: &mut World| {
                        let mut query = world.query::<(Entity, &Name)>();
                        let popup = query
                            .iter(world)
                            .find(|(_, name)| name.as_str() == "floating-popup")
                            .map(|(entity, _)| entity);
                        if let Some(popup) = popup {
                            world
                                .entity_mut(popup)
                                .remove::<ui::floating::FloatingPopup>();
                        }
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("popup kept a margin inside the window"),
                    apply: |world: &mut World| {
                        let mut query = world.query::<(Entity, &Name)>();
                        let popup = query
                            .iter(world)
                            .find(|(_, name)| name.as_str() == "floating-popup")
                            .map(|(entity, _)| entity);
                        if let Some(popup) = popup {
                            world.entity_mut(popup).insert(ui::floating::FloatingPopup {
                                anchor: ui::floating::PopupAnchor::Parent,
                                preferred_side: ui::floating::PopupSide::Below,
                                cross_align: ui::floating::PopupAlign::Centre,
                                gap: ui::floating::PopupGap(4.0),
                                edge_margin: ui::floating::DEFAULT_POPUP_EDGE_MARGIN,
                            });
                        }
                    },
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
