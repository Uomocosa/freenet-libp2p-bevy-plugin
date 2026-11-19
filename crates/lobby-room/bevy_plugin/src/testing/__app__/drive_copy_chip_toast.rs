use bevy::input::keyboard::KeyCode;
use bevy::prelude::{App, Visibility};

use crate::testing;
use crate::ui;

pub fn drive_copy_chip_toast(app: &mut App, drawer_slide_secs: f32) -> Result<(), String> {
    testing::press_key(app, KeyCode::Tab);
    if !testing::tick_until(app, 400, |app| {
        let slide = testing::component::<ui::direct_dial::FriendsDrawerSlide>(app.world_mut())
            .map_or(0.0, |slide| *slide);
        (slide - drawer_slide_secs).abs() < f32::EPSILON
    }) {
        return Err(String::from("the drawer never fully opened"));
    }
    if testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("Tab opens the drawer"));
    }
    testing::press_marker::<ui::direct_dial::OwnAddressChip>(app);
    app.update();
    if testing::marker_text::<ui::direct_dial::CopyToast>(app.world_mut())
        != Some(String::from("Copied to clipboard"))
    {
        return Err(String::from("the copy click shows its toast"));
    }
    testing::press_key(app, KeyCode::Tab);
    if !testing::tick_until(app, 400, |app| {
        testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
            == Some(Visibility::Hidden)
    }) {
        return Err(String::from("the drawer never closed"));
    }
    if testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("Tab closes the drawer"));
    }
    Ok(())
}

// no test_usage necessary
