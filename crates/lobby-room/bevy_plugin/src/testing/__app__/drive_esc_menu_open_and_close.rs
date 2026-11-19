use bevy::input::keyboard::KeyCode;
use bevy::prelude::{App, Visibility};

use crate::testing;
use crate::ui;

pub fn drive_esc_menu_open_and_close(app: &mut App) -> Result<(), String> {
    testing::press_key(app, KeyCode::Escape);
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("Esc opens the menu"));
    }
    testing::press_key(app, KeyCode::Escape);
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("Esc closes the menu"));
    }
    Ok(())
}

// no test_usage necessary
