use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn translate_libp2p_events_to_dial_inputs(
    mut tap: ResMut<direct_dial::DialEventTap>,
    mut inputs: MessageWriter<direct_dial::state_machine::Input>,
) {
    while let Ok(event) = tap.try_recv() {
        match event {
            p2p::Libp2pEvent::PeerConnected(id) => {
                inputs.write(direct_dial::state_machine::Input::PeerConnected(id));
            }
            p2p::Libp2pEvent::PeerDisconnected(id) => {
                inputs.write(direct_dial::state_machine::Input::PeerDisconnected(id));
            }
            p2p::Libp2pEvent::DialFailed { peer_id, .. } => {
                inputs.write(direct_dial::state_machine::Input::DialFailed(peer_id));
            }
            p2p::Libp2pEvent::PeerListenAddrs { peer_id, addrs } => {
                inputs.write(direct_dial::state_machine::Input::PeerListenAddrs(
                    identity::Peer { id: peer_id, addrs },
                ));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::translate_libp2p_events_to_dial_inputs;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    fn app_with_tap() -> (App, tokio::sync::broadcast::Sender<p2p::Libp2pEvent>) {
        let channel = p2p::Libp2pEventChannel::default();
        let tx = channel.tx.clone();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Input>();
        app.insert_resource(channel);
        let rx = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        app.insert_resource(direct_dial::DialEventTap(rx));
        app.add_systems(Update, translate_libp2p_events_to_dial_inputs);
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
        let inputs: Vec<direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![direct_dial::state_machine::Input::PeerConnected(
                identity::PeerId::from("alice")
            )]
        );
    }

    #[test]
    fn dial_failed_maps_to_the_right_peer() {
        let (mut app, tx) = app_with_tap();
        tx.send(p2p::Libp2pEvent::DialFailed {
            peer_id: identity::PeerId::from("alice"),
            reason: String::from("refused"),
        })
        .expect("send");
        app.update();
        let inputs: Vec<direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![direct_dial::state_machine::Input::DialFailed(
                identity::PeerId::from("alice")
            )]
        );
    }

    #[test]
    fn listen_addrs_become_an_input() {
        let (mut app, tx) = app_with_tap();
        let addrs = vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")];
        tx.send(p2p::Libp2pEvent::PeerListenAddrs {
            peer_id: identity::PeerId::from("alice"),
            addrs: addrs.clone(),
        })
        .expect("send");
        app.update();
        let inputs: Vec<direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![direct_dial::state_machine::Input::PeerListenAddrs(
                identity::Peer {
                    id: identity::PeerId::from("alice"),
                    addrs,
                }
            )]
        );
    }

    #[test]
    fn unrelated_events_are_ignored() {
        let (mut app, tx) = app_with_tap();
        tx.send(p2p::Libp2pEvent::RelayReserved {
            relay_peer_id: identity::PeerId::from("relay"),
        })
        .expect("send");
        app.update();
        let pending = app
            .world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(pending, 0);
    }
}
