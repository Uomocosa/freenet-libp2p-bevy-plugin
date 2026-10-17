use bevy::prelude::*;

use crate::p2p;

pub fn poll_p2p_events<T: p2p::Message>(
    mut events: ResMut<p2p::Events<T>>,
    mut commands: ResMut<p2p::Commands<T>>,
    mut pending: ResMut<p2p::PendingNetCommands>,
    command_channel: Res<p2p::CommandSender<T>>,
    event_channel: Res<p2p::EventReceiver<T>>,
    net_commands: Option<Res<p2p::NetCommandSender>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    net_events: Option<Res<p2p::NetEventChannel>>,
) {
    let command_channel = command_channel.into_inner();
    let event_channel = event_channel.into_inner();
    for command in commands.take_all() {
        command_channel.send(command).ok();
    }
    if let Some(net_commands) = net_commands {
        for command in pending.take_all() {
            net_commands.send(command).ok();
        }
    }
    let drained = event_channel.lock().map_or_else(
        |_| Vec::new(),
        |mut guard| {
            let mut drained = Vec::new();
            if let Some(rx) = guard.as_mut() {
                while let Ok(event) = rx.try_recv() {
                    drained.push(event);
                }
            }
            drained
        },
    );
    for event in &drained {
        match event {
            p2p::Event::Net(p2p::NetEvent::Ready(me)) => {
                tracing::info!("ready peer_id={} addrs={:?}", me.id, me.addrs);
            }
            p2p::Event::Error(message) => {
                tracing::warn!("p2p error: {message}");
            }
            _ => {}
        }
    }
    if let Some(ready) = ready {
        for event in &drained {
            publish_ready(&ready, event);
        }
    }
    if let Some(net_events) = net_events {
        for event in &drained {
            if let Some(net_event) = net_event_of(event) {
                net_events.tx.send(net_event).ok();
            }
        }
    }
    events.extend(drained);
}

// needed helper: updates the latest-value ready/observed watch channels
fn publish_ready<T: p2p::Message>(ready: &p2p::ReadyChannel, event: &p2p::Event<T>) {
    match event {
        p2p::Event::Net(p2p::NetEvent::Ready(me)) => {
            ready.ready_tx.send_replace(Some(me.clone()));
        }
        p2p::Event::Net(p2p::NetEvent::ObservedAddr(addr)) => {
            ready.observed_tx.send_replace(Some(vec![addr.clone()]));
        }
        _ => {}
    }
}

// needed helper: maps the non-generic subset of events onto the net event channel
fn net_event_of<T: p2p::Message>(event: &p2p::Event<T>) -> Option<p2p::NetEvent> {
    match event {
        p2p::Event::Net(net_event) => Some(net_event.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::poll_p2p_events;
    use crate::identity;
    use crate::p2p;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command<Dummy>>();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.insert_resource(p2p::Commands::<Dummy>::default());
        app.insert_resource(p2p::PendingNetCommands::default());
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                identity::PeerId::from("peer"),
            )))
            .ok();
        app.world_mut()
            .resource_mut::<p2p::Commands<Dummy>>()
            .push(p2p::Command::Net(p2p::NetCommand::Dial(identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: vec![],
            })));
        app.insert_resource(p2p::CommandSender(cmd_tx));
        app.insert_resource(p2p::EventReceiver(std::sync::Mutex::new(Some(event_rx))));
        app.add_systems(Update, poll_p2p_events::<Dummy>);
        app.update();
        assert!(cmd_rx.try_recv().is_ok());
        assert_eq!(app.world().resource::<p2p::Events<Dummy>>().len(), 1);
    }

    #[test]
    fn net_event_channel_forwards_non_generic_events() {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command<Dummy>>();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let (tap_tx, mut tap_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::NetEvent>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.insert_resource(p2p::Commands::<Dummy>::default());
        app.insert_resource(p2p::PendingNetCommands::default());
        app.insert_resource(p2p::CommandSender(cmd_tx));
        app.insert_resource(p2p::EventReceiver(std::sync::Mutex::new(Some(event_rx))));
        app.insert_resource(p2p::NetEventChannel {
            tx: tap_tx,
            rx: std::sync::Mutex::new(None),
        });
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                identity::PeerId::from("peer"),
            )))
            .ok();
        event_tx
            .send(p2p::Event::Message {
                from: identity::PeerId::from("peer"),
                payload: Dummy(1),
            })
            .ok();
        app.add_systems(Update, poll_p2p_events::<Dummy>);
        app.update();
        let tapped = tap_rx.try_recv();
        assert_eq!(
            tapped,
            Ok(p2p::NetEvent::PeerConnected(identity::PeerId::from("peer")))
        );
        assert!(
            tap_rx.try_recv().is_err(),
            "game messages stay off the net event channel"
        );
    }
}
