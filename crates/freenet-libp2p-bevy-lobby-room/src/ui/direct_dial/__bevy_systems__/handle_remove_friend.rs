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

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_remove_friend;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        let mut book = core_direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        app.insert_resource(core_direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        ));
        app.world_mut().spawn((
            ui::direct_dial::FriendRemoveButton(identity::PeerId::from("alice")),
            Pressed,
        ));
        app.add_systems(Update, handle_remove_friend);
        app.update();
        let inputs: Vec<core_direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<core_direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![core_direct_dial::state_machine::Input::FriendRemoved(
                identity::PeerId::from("alice")
            )]
        );
    }
}
