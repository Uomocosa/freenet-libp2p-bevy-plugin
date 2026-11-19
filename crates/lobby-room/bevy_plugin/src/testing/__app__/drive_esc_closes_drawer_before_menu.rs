use bevy::input::keyboard::KeyCode;
use bevy::prelude::{App, Visibility};

use crate::testing;
use crate::ui;

pub fn drive_esc_closes_drawer_before_menu(
    app: &mut App,
    drawer_slide_secs: f32,
) -> Result<(), String> {
    testing::press_key(app, KeyCode::Tab);
    if !testing::tick_until(app, 400, |app| {
        let slide = testing::component::<ui::direct_dial::FriendsDrawerSlide>(app.world_mut())
            .map_or(0.0, |slide| *slide);
        (slide - drawer_slide_secs).abs() < f32::EPSILON
    }) {
        return Err(String::from("the drawer never fully opened"));
    }
    testing::press_key(app, KeyCode::Escape);
    if !testing::tick_until(app, 400, |app| {
        testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
            == Some(Visibility::Hidden)
    }) {
        return Err(String::from("Esc never closed the drawer"));
    }
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("Esc closes the drawer before the menu"));
    }
    testing::press_key(app, KeyCode::Escape);
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("Esc then opens the menu"));
    }
    testing::press_key(app, KeyCode::Escape);
    Ok(())
}

// no test_usage necessary
