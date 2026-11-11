use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn update_own_share_address(
    ready: Option<Res<p2p::ReadyChannel>>,
    mut share: ResMut<direct_dial::OwnShareAddress>,
) {
    let address = ready.and_then(|ready| {
        let me = ready.ready_rx.borrow().as_ref().cloned()?;
        let observed: Vec<identity::PeerAddr> = ready
            .observed_rx
            .borrow()
            .as_ref()
            .map(|addrs| addrs.as_slice().to_vec())
            .unwrap_or_default();
        Some(direct_dial::OwnAddress::from_own_peer(&me, &observed))
    });
    **share = address;
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_own_share_address;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    fn ready_with(addrs: Vec<&str>) -> p2p::ReadyChannel {
        let channel = p2p::ReadyChannel::default();
        channel.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"),
            addrs: addrs.into_iter().map(identity::PeerAddr::from).collect(),
        }));
        channel
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<direct_dial::OwnShareAddress>();
        app.insert_resource(ready_with(vec!["/ip4/127.0.0.1/tcp/4001"]));
        app.add_systems(Update, update_own_share_address);
        app.update();
        let share = app.world().resource::<direct_dial::OwnShareAddress>();
        assert_eq!(
            (**share).as_ref().map(|address| address.short.clone()),
            Some(String::from("12D3Koo...99RC"))
        );
        assert!(
            (**share)
                .as_ref()
                .is_some_and(|address| address.full.is_some())
        );
    }

    #[test]
    fn without_a_ready_channel_the_share_is_empty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<direct_dial::OwnShareAddress>();
        app.add_systems(Update, update_own_share_address);
        app.update();
        let share = app.world().resource::<direct_dial::OwnShareAddress>();
        assert_eq!(**share, None);
    }

    #[test]
    fn ready_without_an_address_still_shows_the_short_id() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<direct_dial::OwnShareAddress>();
        app.insert_resource(ready_with(Vec::new()));
        app.add_systems(Update, update_own_share_address);
        app.update();
        let share = app.world().resource::<direct_dial::OwnShareAddress>();
        let address = (**share).as_ref().expect("ready peer");
        assert_eq!(address.full, None);
        assert_eq!(address.short, "12D3Koo...99RC");
    }
}
