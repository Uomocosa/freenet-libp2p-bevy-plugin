use bevy::prelude::*;

use crate::discovery;

pub fn refresh_status_from_discovery_state(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    status: Option<ResMut<discovery::Status>>,
) {
    let Some(state) = state else {
        return;
    };
    let Some(mut status) = status else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    *status = state.status();
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::refresh_status_from_discovery_state;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut state = discovery::state_machine::DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.is_lobby_fetched = true;
        app.insert_resource(state);
        app.init_resource::<discovery::Status>();
        app.add_systems(Update, refresh_status_from_discovery_state);
        app.update();
        assert!(app.world().resource::<discovery::Status>().is_lobby_fetched);
        let _ = time::UnixTime::EPOCH;
    }

    #[test]
    fn unchanged_state_keeps_old_status() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::state_machine::DiscoveryState::new(
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: Vec::new(),
            },
        ));
        app.init_resource::<discovery::Status>();
        app.add_systems(Update, refresh_status_from_discovery_state);
        app.update();
        app.update();
        assert!(!app.world().resource::<discovery::Status>().is_lobby_fetched);
    }
}
