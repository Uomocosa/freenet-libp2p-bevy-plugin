use bevy::prelude::*;

use crate::direct_dial;
use crate::p2p;

pub fn dial_outputs_to_libp2p_commands(
    mut outputs: MessageReader<direct_dial::state_machine::Output>,
    mut pending: ResMut<p2p::PendingLibp2pCommands>,
) {
    for output in outputs.read() {
        if let direct_dial::state_machine::Output::Dial(peer) = output {
            pending.push(p2p::Libp2pCommand::Dial(peer.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::dial_outputs_to_libp2p_commands;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Output>();
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.add_systems(Update, dial_outputs_to_libp2p_commands);
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Output>>()
            .write(direct_dial::state_machine::Output::Dial(identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            }));
        app.update();
        let pending = app.world().resource::<p2p::PendingLibp2pCommands>();
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.first(),
            Some(p2p::Libp2pCommand::Dial(peer)) if peer.id == identity::PeerId::from("alice")
        ));
    }

    #[test]
    fn save_outputs_push_no_command() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Output>();
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.add_systems(Update, dial_outputs_to_libp2p_commands);
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Output>>()
            .write(direct_dial::state_machine::Output::SavePeerBook(
                direct_dial::PeerBook::default(),
            ));
        app.update();
        assert!(
            app.world()
                .resource::<p2p::PendingLibp2pCommands>()
                .is_empty()
        );
    }
}
