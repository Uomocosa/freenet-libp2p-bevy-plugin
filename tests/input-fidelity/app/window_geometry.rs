use bevy::prelude::*;
use bevy::window::WindowPosition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

impl WindowGeometry {
    #[must_use]
    pub fn from_window(window: &Window) -> Self {
        let (x, y) = match window.position {
            WindowPosition::At(position) => (Some(position.x), Some(position.y)),
            WindowPosition::Automatic | WindowPosition::Centered(_) => (None, None),
        };
        Self {
            x,
            y,
            width: window.physical_width(),
            height: window.physical_height(),
            scale_factor: window.scale_factor(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::WindowPosition;

    use super::WindowGeometry;

    #[test]
    fn test_usage() {
        let window = Window {
            position: WindowPosition::At(IVec2::new(0, 0)),
            ..Default::default()
        };
        let geometry = WindowGeometry::from_window(&window);
        assert_eq!(geometry.x, Some(0));
        assert_eq!(geometry.y, Some(0));
        assert!(geometry.width > 0);
        assert!(geometry.height > 0);
        let json = serde_json::to_value(geometry).expect("geometry serialises");
        let back: WindowGeometry = serde_json::from_value(json).expect("geometry round-trips");
        assert_eq!(back, geometry);
    }
}
