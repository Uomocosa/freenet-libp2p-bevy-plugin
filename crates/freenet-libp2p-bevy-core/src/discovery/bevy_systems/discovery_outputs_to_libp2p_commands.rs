use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn discovery_outputs_to_libp2p_commands(
    mut outputs: MessageReader<discovery::state_machine::Output>,
    pending: Option<ResMut<p2p::PendingLibp2pCommands>>,
) {
    let Some(mut pending) = pending else {
        return;
    };
    for output in outputs.read() {
        if let discovery::state_machine::Output::Libp2pCommand(command) = output {
            pending.push(command.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::discovery_outputs_to_libp2p_commands;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Output>();
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.add_systems(Update, discovery_outputs_to_libp2p_commands);
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Output>>()
            .write(discovery::state_machine::Output::Libp2pCommand(
                p2p::Libp2pCommand::Dial(identity::Peer {
                    id: identity::PeerId::from("alice"),
                    addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
                }),
            ));
        app.update();
        let pending = app.world().resource::<p2p::PendingLibp2pCommands>();
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.first(),
            Some(p2p::Libp2pCommand::Dial(peer)) if peer.id == identity::PeerId::from("alice")
        ));
    }

    #[test]
    fn notify_outputs_push_no_command() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Output>();
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.add_systems(Update, discovery_outputs_to_libp2p_commands);
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Output>>()
            .write(discovery::state_machine::Output::Notify(
                discovery::Event::LobbyUpdated,
            ));
        app.update();
        assert!(
            app.world()
                .resource::<p2p::PendingLibp2pCommands>()
                .is_empty()
        );
    }
}
