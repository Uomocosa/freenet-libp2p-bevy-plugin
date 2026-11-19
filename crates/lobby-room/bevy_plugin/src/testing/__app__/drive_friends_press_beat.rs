use bevy::input::keyboard::KeyCode;
use bevy::prelude::{App, Visibility};
use bevy::ui::Pressed;

use crate::testing;
use crate::ui;

pub fn drive_friends_press_beat(app: &mut App) -> Result<(), String> {
    testing::press_key(app, KeyCode::Escape);
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("Esc opens the menu"));
    }
    testing::press_marker::<ui::discovery::EscMenuFriendsButton>(app);
    app.update();
    if testing::find_entity::<ui::discovery::EscMenuFriendsButton>(app.world_mut())
        .is_none_or(|button| app.world().get::<Pressed>(button).is_none())
    {
        return Err(String::from(
            "the Friends button keeps its pressed style during the beat",
        ));
    }
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Visible)
    {
        return Err(String::from("the beat keeps the menu open"));
    }
    if testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("the beat keeps the drawer closed"));
    }
    if !testing::tick_until(app, 400, |app| {
        testing::marker_visibility::<ui::direct_dial::FriendsDrawer>(app.world_mut())
            == Some(Visibility::Visible)
    }) {
        return Err(String::from("the drawer never opened"));
    }
    if testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut())
        != Some(Visibility::Hidden)
    {
        return Err(String::from("the beat ends by closing the menu"));
    }
    if testing::find_entity::<ui::discovery::EscMenuFriendsButton>(app.world_mut())
        .is_some_and(|button| app.world().get::<Pressed>(button).is_some())
    {
        return Err(String::from("the pressed style clears once the beat fires"));
    }
    if !testing::component::<ui::direct_dial::FriendsDrawerSlide>(app.world_mut())
        .is_some_and(|slide| *slide > 0.0)
    {
        return Err(String::from("the beat ends by opening the drawer"));
    }
    Ok(())
}

// no test_usage necessary
