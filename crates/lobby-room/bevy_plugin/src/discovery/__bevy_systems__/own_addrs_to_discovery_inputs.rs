use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;

pub fn own_addrs_to_discovery_inputs(
    ready: Option<ResMut<p2p::ReadyChannel>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(mut ready) = ready else {
        return;
    };
    let Ok(has_changed) = ready.observed_rx.has_changed() else {
        return;
    };
    if !has_changed {
        return;
    }
    let Some(addrs) = ready.observed_rx.borrow_and_update().clone() else {
        return;
    };
    inputs.write(discovery::state_machine::Input::OwnAddrsChanged(
        identity::PeerAddr::dialable(addrs),
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::own_addrs_to_discovery_inputs;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Input>();
        let channel = p2p::ReadyChannel::default();
        channel
            .observed_tx
            .send_replace(Some(vec![identity::PeerAddr::from(
                "/ip4/1.2.3.4/tcp/4001",
            )]));
        app.insert_resource(channel);
        app.add_systems(Update, own_addrs_to_discovery_inputs);
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::OwnAddrsChanged(vec![
                identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")
            ])]
        );
    }

    #[test]
    fn no_change_writes_no_input() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Input>();
        app.insert_resource(p2p::ReadyChannel::default());
        app.add_systems(Update, own_addrs_to_discovery_inputs);
        app.update();
        let count = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(count, 0);
    }
}
