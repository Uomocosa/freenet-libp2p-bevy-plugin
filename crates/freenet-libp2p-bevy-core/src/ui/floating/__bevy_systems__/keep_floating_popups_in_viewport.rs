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
    use bevy::math::Vec2;
    use bevy::prelude::*;
    use bevy::ui::ComputedNode;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use super::keep_floating_popups_in_viewport;
    use crate::ui;

    fn px(value: Val) -> f32 {
        match value {
            Val::Px(px) => px,
            _ => panic!("expected a px value"),
        }
    }

    fn popup_component(anchor: ui::floating::PopupAnchor) -> ui::floating::FloatingPopup {
        ui::floating::FloatingPopup {
            anchor,
            preferred_side: ui::floating::PopupSide::Below,
            cross_align: ui::floating::PopupAlign::Centre,
            gap: ui::floating::PopupGap(4.0),
            edge_margin: ui::floating::DEFAULT_POPUP_EDGE_MARGIN,
        }
    }

    fn spawn_window(app: &mut App) {
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(960, 540),
                ..default()
            },
            PrimaryWindow,
        ));
    }

    fn spawn_anchor(app: &mut App, size: Vec2, inverse: f32, at: Vec2) -> Entity {
        let mut node = ComputedNode {
            size,
            ..ComputedNode::default()
        };
        node.inverse_scale_factor = inverse;
        app.world_mut()
            .spawn((node, UiGlobalTransform::from_xy(at.x, at.y)))
            .id()
    }

    fn spawn_popup(
        app: &mut App,
        anchor: ui::floating::PopupAnchor,
        size: Vec2,
        inverse: f32,
        parent: Option<Entity>,
    ) -> Entity {
        let mut node = ComputedNode {
            size,
            ..ComputedNode::default()
        };
        node.inverse_scale_factor = inverse;
        let mut entity = app.world_mut().spawn((
            popup_component(anchor),
            node,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        ));
        if let Some(parent) = parent {
            entity.insert(ChildOf(parent));
        }
        entity.id()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        spawn_window(&mut app);
        let anchor = spawn_anchor(
            &mut app,
            Vec2::new(50.0, 20.0),
            1.0,
            Vec2::new(920.0, 100.0),
        );
        spawn_popup(
            &mut app,
            ui::floating::PopupAnchor::Parent,
            Vec2::new(150.0, 18.0),
            1.0,
            Some(anchor),
        );
        app.add_systems(Update, keep_floating_popups_in_viewport);
        app.update();
    }

    #[test]
    fn a_popup_near_the_right_edge_is_moved_back_inside() {
        let mut app = App::new();
        spawn_window(&mut app);
        let anchor = spawn_anchor(
            &mut app,
            Vec2::new(50.0, 20.0),
            1.0,
            Vec2::new(920.0, 100.0),
        );
        let popup = spawn_popup(
            &mut app,
            ui::floating::PopupAnchor::Parent,
            Vec2::new(150.0, 18.0),
            1.0,
            Some(anchor),
        );
        app.add_systems(Update, keep_floating_popups_in_viewport);
        app.update();
        let node = app.world().get::<Node>(popup).expect("popup node");
        assert_eq!(node.left, Val::Px(-93.0));
        assert_eq!(node.top, Val::Px(24.0));
        let anchor_left = 920.0 - 50.0 / 2.0;
        assert!(
            anchor_left + px(node.left) + 150.0 <= 960.0 - 8.0,
            "the popup keeps the edge margin"
        );
    }

    #[test]
    fn a_popup_with_no_measured_size_is_left_alone() {
        let mut app = App::new();
        spawn_window(&mut app);
        let anchor = spawn_anchor(
            &mut app,
            Vec2::new(50.0, 20.0),
            1.0,
            Vec2::new(920.0, 100.0),
        );
        let popup = spawn_popup(
            &mut app,
            ui::floating::PopupAnchor::Parent,
            Vec2::ZERO,
            1.0,
            Some(anchor),
        );
        app.add_systems(Update, keep_floating_popups_in_viewport);
        app.update();
        let node = app.world().get::<Node>(popup).expect("popup node");
        assert_eq!(node.left, Val::Auto);
        assert_eq!(node.top, Val::Auto);
    }

    #[test]
    fn a_top_level_popup_uses_window_coordinates() {
        let mut app = App::new();
        spawn_window(&mut app);
        let anchor = spawn_anchor(
            &mut app,
            Vec2::new(50.0, 20.0),
            1.0,
            Vec2::new(920.0, 100.0),
        );
        let popup = spawn_popup(
            &mut app,
            ui::floating::PopupAnchor::Entity(anchor),
            Vec2::new(150.0, 18.0),
            1.0,
            None,
        );
        app.add_systems(Update, keep_floating_popups_in_viewport);
        app.update();
        let node = app.world().get::<Node>(popup).expect("popup node");
        assert_eq!(node.left, Val::Px(802.0));
        assert_eq!(node.top, Val::Px(114.0));
    }

    #[test]
    fn a_two_times_inverse_scale_factor_is_converted_to_logical_pixels() {
        let mut app = App::new();
        spawn_window(&mut app);
        let anchor = spawn_anchor(
            &mut app,
            Vec2::new(50.0, 20.0),
            2.0,
            Vec2::new(400.0, 100.0),
        );
        let popup = spawn_popup(
            &mut app,
            ui::floating::PopupAnchor::Parent,
            Vec2::new(150.0, 18.0),
            2.0,
            Some(anchor),
        );
        app.add_systems(Update, keep_floating_popups_in_viewport);
        app.update();
        let node = app.world().get::<Node>(popup).expect("popup node");
        assert_eq!(node.left, Val::Px(-100.0));
        assert_eq!(node.top, Val::Px(44.0));
    }
}

#[cfg(test)]
mod png_tests {
    use bevy::prelude::*;

    use super::keep_floating_popups_in_viewport;
    use crate::ui;

    const POPUP_NAME: &str = "floating-popup";

    fn popup_component() -> ui::floating::FloatingPopup {
        ui::floating::FloatingPopup {
            anchor: ui::floating::PopupAnchor::Parent,
            preferred_side: ui::floating::PopupSide::Below,
            cross_align: ui::floating::PopupAlign::Centre,
            gap: ui::floating::PopupGap(4.0),
            edge_margin: ui::floating::DEFAULT_POPUP_EDGE_MARGIN,
        }
    }

    fn popup_entity(world: &mut World) -> Option<Entity> {
        let mut query = world.query::<(Entity, &Name)>();
        query
            .iter(world)
            .find(|(_, name)| name.as_str() == POPUP_NAME)
            .map(|(entity, _)| entity)
    }

    fn spawn_scene(mut commands: Commands) {
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
            Name::new(POPUP_NAME),
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
    }

    fn floating_disabled(world: &mut World) {
        if let Some(popup) = popup_entity(world) {
            world
                .entity_mut(popup)
                .remove::<ui::floating::FloatingPopup>();
        }
    }

    fn floating_enabled(world: &mut World) {
        if let Some(popup) = popup_entity(world) {
            world.entity_mut(popup).insert(popup_component());
        }
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_scene);
        app.add_systems(Update, keep_floating_popups_in_viewport);
    }

    #[test]
    fn keep_floating_popups_in_viewport_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("keep_floating_popups_in_viewport"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("popup overflowing the right edge"),
                    apply: floating_disabled,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("popup kept a margin inside the window"),
                    apply: floating_enabled,
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
