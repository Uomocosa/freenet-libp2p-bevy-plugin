use bevy::input::keyboard::KeyCode;
use bevy::prelude::{App, Visibility, With};

use crate::testing;
use crate::ui;

pub fn drive_esc_closes_leave_prompt(app: &mut App) -> Result<(), String> {
    testing::press_key(app, KeyCode::Escape);
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("Esc opens the menu in the room"));
    }
    testing::press_marker::<ui::discovery::EscMenuLeaveButton>(app);
    if !testing::tick_until(app, 400, |app| {
        testing::count_matching::<(), With<ui::discovery::LeavePrompt>>(app.world_mut()) == 1
    }) {
        return Err(String::from("the leave prompt never opened"));
    }
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("the leave press closes the menu"));
    }
    testing::press_key(app, KeyCode::Escape);
    if !testing::tick_until(app, 400, |app| {
        testing::count_matching::<(), With<ui::discovery::LeavePrompt>>(app.world_mut()) == 0
    }) {
        return Err(String::from("the leave prompt never closed"));
    }
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("Esc closes the leave prompt before the menu"));
    }
    if testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from(
            "Esc closes the leave prompt before the drawer",
        ));
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
