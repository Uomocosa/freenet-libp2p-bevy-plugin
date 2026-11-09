use bevy::prelude::*;

use crate::p2p;

pub fn flush_pending_libp2p_commands(
    pending: Option<ResMut<p2p::PendingLibp2pCommands>>,
    sender: Option<Res<p2p::Libp2pCommandSender>>,
) {
    let Some(mut pending) = pending else {
        return;
    };
    let Some(sender) = sender.map(Res::into_inner) else {
        return;
    };
    for command in pending.take_all() {
        sender.send(command).ok();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::flush_pending_libp2p_commands;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut pending = p2p::PendingLibp2pCommands::default();
        pending.push(p2p::Libp2pCommand::Dial(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        app.insert_resource(pending);
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        app.add_systems(Update, flush_pending_libp2p_commands);
        app.update();
        assert!(rx.try_recv().is_ok());
        assert!(
            app.world()
                .resource::<p2p::PendingLibp2pCommands>()
                .is_empty()
        );
    }

    #[test]
    fn empty_pending_sends_nothing() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        app.add_systems(Update, flush_pending_libp2p_commands);
        app.update();
        assert!(rx.try_recv().is_err());
    }
}
