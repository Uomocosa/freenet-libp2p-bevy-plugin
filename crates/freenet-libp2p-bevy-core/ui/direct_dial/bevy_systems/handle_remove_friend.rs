use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::ui;
use ui::direct_dial::FriendRemoveButton;

pub fn handle_remove_friend(
    buttons: Query<&FriendRemoveButton, Added<Pressed>>,
    mut inputs: MessageWriter<direct_dial::state_machine::Input>,
) {
    for button in &buttons {
        inputs.write(direct_dial::state_machine::Input::FriendRemoved(
            (**button).clone(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_remove_friend;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Input>();
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        ));
        app.world_mut().spawn((
            ui::direct_dial::FriendRemoveButton(identity::PeerId::from("alice")),
            Pressed,
        ));
        app.add_systems(Update, handle_remove_friend);
        app.update();
        let inputs: Vec<direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![direct_dial::state_machine::Input::FriendRemoved(
                identity::PeerId::from("alice")
            )]
        );
    }
}
