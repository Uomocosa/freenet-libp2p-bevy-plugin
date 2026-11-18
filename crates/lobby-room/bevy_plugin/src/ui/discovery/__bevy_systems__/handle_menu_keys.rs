use crate::ui;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{
    EscMenuFriendsButton, EscMenuState, FriendsPressBeat, LeaveDialog, StreamDisplay,
};

pub fn handle_menu_keys(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut menu: ResMut<EscMenuState>,
    mut beat: ResMut<FriendsPressBeat>,
    mut drawer: Option<ResMut<ui::direct_dial::FriendsDrawerState>>,
    mut display: Option<ResMut<StreamDisplay>>,
    dialog: Option<Res<State<LeaveDialog>>>,
    next_dialog: Option<ResMut<NextState<LeaveDialog>>>,
    friends: Query<Entity, With<EscMenuFriendsButton>>,
    mut commands: Commands,
) {
    let Some(keys) = keys else {
        return;
    };
    let keys = keys.into_inner();
    if keys.just_pressed(KeyCode::Escape) {
        **beat = 0.0;
        let is_fullscreen = display
            .as_deref()
            .is_some_and(|display| *display == StreamDisplay::Fullscreen);
        let is_drawer_open = drawer.as_deref().is_some_and(|drawer| **drawer);
        let is_dialog_open = dialog.is_some_and(|dialog| **dialog == LeaveDialog::Open);
        if is_fullscreen {
            if let Some(display) = display.as_deref_mut() {
                display.show_panel();
            }
        } else if is_drawer_open {
            if let Some(drawer) = drawer.as_deref_mut() {
                drawer.close();
            }
        } else if is_dialog_open {
            if let Some(mut next_dialog) = next_dialog {
                next_dialog.set(LeaveDialog::Closed);
            }
        } else {
            menu.toggle();
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        if **menu {
            for entity in &friends {
                commands.entity(entity).insert(Pressed);
            }
        } else if let Some(drawer) = drawer.as_deref_mut() {
            drawer.toggle();
        }
    }
}
