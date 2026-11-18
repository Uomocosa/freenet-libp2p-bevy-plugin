use bevy::prelude::*;

use crate::p2p;

pub fn poll_p2p_events<Payload: p2p::traits::Message>(
    mut events: ResMut<p2p::PendingEvents<Payload>>,
    mut commands: ResMut<p2p::PendingCommands<Payload>>,
    command_channel: Res<p2p::CommandSender<Payload>>,
    event_channel: Res<p2p::EventReceiver<Payload>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    libp2p_events: Option<Res<p2p::Libp2pEventChannel>>,
) {
    let command_channel = command_channel.into_inner();
    let event_channel = event_channel.into_inner();
    for command in commands.take_all() {
        command_channel.send(command).ok();
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
            p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(me)) => {
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
            match event {
                p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(me)) => {
                    ready.ready_tx.send_replace(Some(me.clone()));
                }
                p2p::Event::Libp2p(p2p::Libp2pEvent::ObservedAddr(addr)) => {
                    ready.observed_tx.send_replace(Some(vec![addr.clone()]));
                }
                _ => {}
            }
        }
    }
    if let Some(libp2p_events) = libp2p_events {
        for event in &drained {
            if let p2p::Event::Libp2p(libp2p_event) = event {
                libp2p_events.send(libp2p_event.clone());
            }
        }
    }
    events.extend(drained);
}
