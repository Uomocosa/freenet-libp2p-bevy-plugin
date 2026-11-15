use bevy::prelude::*;

use crate::discovery;

pub fn translate_libp2p_events_to_discovery_inputs(
    mut tap: Option<ResMut<discovery::Libp2pEventTap>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(tap) = tap.as_deref_mut() else {
        return;
    };
    while let Ok(event) = tap.try_recv() {
        inputs.write(discovery::state_machine::Input::Libp2pEvent(event));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::translate_libp2p_events_to_discovery_inputs;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    fn app_with_tap() -> (App, tokio::sync::broadcast::Sender<p2p::Libp2pEvent>) {
        let channel = p2p::Libp2pEventChannel::default();
        let tx = channel.tx.clone();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Input>();
        app.insert_resource(channel);
        let rx = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        app.insert_resource(discovery::Libp2pEventTap(rx));
        app.add_systems(Update, translate_libp2p_events_to_discovery_inputs);
        (app, tx)
    }

    #[test]
    fn test_usage() {
        let (mut app, tx) = app_with_tap();
        tx.send(p2p::Libp2pEvent::PeerConnected(identity::PeerId::from(
            "alice",
        )))
        .expect("send");
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::Libp2pEvent(
                p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("alice"))
            )]
        );
    }

    #[test]
    fn unrelated_events_are_forwarded_for_the_machine_to_ignore() {
        let (mut app, tx) = app_with_tap();
        tx.send(p2p::Libp2pEvent::RelayReserved {
            relay_peer_id: identity::PeerId::from("relay"),
        })
        .expect("send");
        app.update();
        let count = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(count, 1);
    }
}
