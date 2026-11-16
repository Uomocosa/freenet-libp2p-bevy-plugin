use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use bevy::window::PrimaryWindow;
use serde_json::Value;

use crate::window_geometry::WindowGeometry;

pub fn probe_window(
    In(_params): In<Option<Value>>,
    window: Query<&Window, With<PrimaryWindow>>,
) -> BrpResult {
    let window = window.single().map_err(BrpError::internal)?;
    serde_json::to_value(WindowGeometry::from_window(window)).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::{PrimaryWindow, WindowPosition};
    use serde_json::Value;

    use super::probe_window;
    use crate::window_geometry::WindowGeometry;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let window = Window {
            position: WindowPosition::At(IVec2::new(0, 0)),
            ..Default::default()
        };
        app.world_mut().spawn((window, PrimaryWindow));
        let system = app.world_mut().register_system(probe_window);
        let result = app.world_mut().run_system_with(system, None::<Value>);
        let Ok(Ok(value)) = result else {
            panic!("probe_window did not run: {result:?}");
        };
        let geometry: WindowGeometry =
            serde_json::from_value(value).expect("window geometry deserialises");
        assert_eq!(geometry.x, Some(0));
        assert_eq!(geometry.y, Some(0));
    }
}
