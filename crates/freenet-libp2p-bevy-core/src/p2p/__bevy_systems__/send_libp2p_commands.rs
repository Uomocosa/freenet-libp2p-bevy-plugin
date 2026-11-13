use bevy::prelude::*;

use crate::p2p;

pub fn send_libp2p_commands(
    mut commands: MessageReader<p2p::Libp2pCommand>,
    sender: Option<Res<p2p::Libp2pCommandSender>>,
) {
    let Some(sender) = sender else {
        commands.clear();
        return;
    };
    for command in commands.read() {
        sender.send(command.clone()).ok();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_libp2p_commands;
    use crate::identity;
    use crate::p2p;

    fn dial(name: &str) -> p2p::Libp2pCommand {
        p2p::Libp2pCommand::Dial(identity::Peer {
            id: identity::PeerId::from(name),
            addrs: Vec::new(),
        })
    }

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        app.add_systems(Update, send_libp2p_commands);
        app.world_mut()
            .resource_mut::<Messages<p2p::Libp2pCommand>>()
            .write(dial("alice"));
        app.update();
        app.update();
        assert_eq!(rx.try_recv().ok(), Some(dial("alice")));
        assert!(rx.try_recv().is_err(), "each command is sent exactly once");
    }

    #[test]
    fn no_messages_sends_nothing() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        app.add_systems(Update, send_libp2p_commands);
        app.update();
        assert!(rx.try_recv().is_err());
    }
}
