use bevy::prelude::*;

use crate::ui;

pub struct UiPlugin;

impl bevy::prelude::Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ui::floating::bevy_systems::keep_floating_popups_in_viewport,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;
    use bevy::prelude::*;
    use bevy::ui::ComputedNode;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use super::UiPlugin;
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(UiPlugin);
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(960, 540),
                ..default()
            },
            PrimaryWindow,
        ));
        let anchor = app
            .world_mut()
            .spawn((
                ComputedNode {
                    size: Vec2::new(50.0, 20.0),
                    ..ComputedNode::default()
                },
                UiGlobalTransform::from_xy(920.0, 100.0),
            ))
            .id();
        let popup = app
            .world_mut()
            .spawn((
                ui::floating::FloatingPopup {
                    anchor: ui::floating::PopupAnchor::Parent,
                    preferred_side: ui::floating::PopupSide::Below,
                    cross_align: ui::floating::PopupAlign::Centre,
                    gap: ui::floating::PopupGap(4.0),
                    edge_margin: ui::floating::DEFAULT_POPUP_EDGE_MARGIN,
                },
                ComputedNode {
                    size: Vec2::new(150.0, 18.0),
                    ..ComputedNode::default()
                },
                ChildOf(anchor),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ))
            .id();
        app.update();
        let node = app.world().get::<Node>(popup).expect("popup node");
        assert_eq!(node.left, Val::Px(-93.0));
    }
}
