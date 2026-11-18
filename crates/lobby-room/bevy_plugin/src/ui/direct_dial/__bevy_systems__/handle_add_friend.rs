use crate::ui;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
use ui::direct_dial::{AddFriendButton, FriendAddressInput};

pub fn handle_add_friend(
    buttons: Query<(), (Added<Pressed>, With<AddFriendButton>)>,
    input: Single<&EditableText, With<FriendAddressInput>>,
    mut submitted: MessageWriter<core_direct_dial::FriendAddressSubmitted>,
) {
    if buttons.is_empty() {
        return;
    }
    submitted.write(core_direct_dial::FriendAddressSubmitted(
        input.into_inner().value().to_string(),
    ));
}
