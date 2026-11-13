use bevy::prelude::*;
use lele_bevy_state_machine::StateMachineSystems;

use crate::discovery;
use crate::presence;

pub fn publish_presence(app: &mut App) {
    app.init_resource::<presence::PeerPresence>();
    app.configure_sets(
        Update,
        presence::PresenceSet::Publish.after(StateMachineSystems::perform_outputs::<
            discovery::state_machine::DiscoveryState,
        >()),
    );
    app.add_systems(
        Update,
        discovery::bevy_systems::publish_peer_presence.in_set(presence::PresenceSet::Publish),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::publish_presence;
    use crate::presence;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        publish_presence(&mut app);
        assert!(
            app.world()
                .get_resource::<presence::PeerPresence>()
                .is_some()
        );
    }
}
