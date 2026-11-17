use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use freenet_libp2p_bevy_plugin::lobby_room::ui;
use freenet_libp2p_bevy_plugin::testing;
use serde_json::Value;

pub fn starboard_stream_close_centre(
    In(_params): In<Option<Value>>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<ui::discovery::RoomStreamCloseButton>>,
) -> BrpResult {
    let centre = buttons
        .iter()
        .next()
        .map(|(node, transform)| *testing::NodeCentre::from_node(node, transform));
    serde_json::to_value(centre).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::remote::RemotePlugin;
    use bevy::ui::{ComputedNode, UiGlobalTransform};
    use freenet_libp2p_bevy_plugin::lobby_room::ui;

    fn centre(app: &mut App) -> Option<Vec2> {
        let system = app
            .world_mut()
            .register_system(super::starboard_stream_close_centre);
        let result = app.world_mut().run_system_with(system, None);
        result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok())
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RemotePlugin::default().with_method_main(
                "starboard/stream_close_centre",
                super::starboard_stream_close_centre,
            ));
        assert_eq!(centre(&mut app), None);
    }

    #[test]
    fn the_close_button_reports_its_centre() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn((
            ui::discovery::RoomStreamCloseButton,
            ComputedNode {
                inverse_scale_factor: 1.0,
                ..ComputedNode::default()
            },
            UiGlobalTransform::from_xy(11.0, 31.0),
        ));
        assert_eq!(centre(&mut app), Some(Vec2::new(11.0, 31.0)));
    }
}
