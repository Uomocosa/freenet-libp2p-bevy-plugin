use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
use ui::direct_dial::FriendRemoveButton;

pub fn handle_remove_friend(
    buttons: Query<&FriendRemoveButton, Added<Pressed>>,
    mut inputs: MessageWriter<core_direct_dial::state_machine::Input>,
) {
    for button in &buttons {
        inputs.write(core_direct_dial::state_machine::Input::FriendRemoved(
            (**button).clone(),
        ));
    }
}
