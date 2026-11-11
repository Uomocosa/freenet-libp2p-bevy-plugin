use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use crate::direct_dial;
use direct_dial::{AddFriendButton, FriendAddressInput};
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::Pressed;

    use super::handle_add_friend;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

    fn app_with(typed: &str, pressed: bool) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::FriendAddressSubmitted>();
        app.world_mut()
            .spawn((direct_dial::FriendAddressInput, EditableText::new(typed)));
        let mut button = app.world_mut().spawn(direct_dial::AddFriendButton);
        if pressed {
            button.insert(Pressed);
        }
        app.add_systems(Update, handle_add_friend);
        app
    }

    fn drained(app: &mut App) -> Vec<core_direct_dial::FriendAddressSubmitted> {
        app.world_mut()
            .resource_mut::<Messages<core_direct_dial::FriendAddressSubmitted>>()
            .drain()
            .collect()
    }

    #[test]
    fn test_usage() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(&format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}"), true);
        app.update();
        let submitted = drained(&mut app);
        assert_eq!(submitted.len(), 1);
        assert_eq!(**submitted[0], format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}"));
    }

    #[test]
    fn an_unpressed_button_submits_nothing() {
        let mut app = app_with("some address", false);
        app.update();
        assert_eq!(drained(&mut app), Vec::new());
    }

    #[test]
    fn a_pressed_button_submits_the_text_even_when_blank() {
        let mut app = app_with("   ", true);
        app.update();
        assert_eq!(drained(&mut app).len(), 1);
    }
}
