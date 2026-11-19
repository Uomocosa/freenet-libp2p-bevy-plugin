use bevy::prelude::*;
use lele_bevy_state_machine::Now;

use crate::discovery;
use crate::p2p;
use crate::time;

pub fn discovery_app_with_swarm() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(p2p::ReadyChannel::default());
    app.insert_resource(p2p::Libp2pEventChannel::default());
    let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(p2p::Libp2pCommandSender(cmd_tx));
    let frozen = time::UnixTime::from_secs(2_000_000_000);
    app.insert_resource(Now(frozen));
    app.add_systems(Last, p2p::bevy_systems::send_libp2p_commands);
    let plugin = discovery::Plugin::new(discovery::Config::default());
    Plugin::build(&plugin, &mut app);
    app
}

#[cfg(test)]
mod tests {
    use crate::p2p;

    use super::discovery_app_with_swarm;

    #[test]
    fn test_usage() {
        let mut app = discovery_app_with_swarm();
        app.update();
        assert!(
            app.world()
                .get_resource::<p2p::Libp2pCommandSender>()
                .is_some()
        );
    }
}
