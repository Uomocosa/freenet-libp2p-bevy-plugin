use bevy::prelude::App;

use crate::p2p;

pub fn swarm_channels(app: &App) -> Option<p2p::SwarmChannels> {
    let events = app
        .world()
        .get_resource::<p2p::Libp2pEventChannel>()?
        .subscribe();
    let commands = (**app.world().get_resource::<p2p::Libp2pCommandSender>()?).clone();
    let (ready, observed) = app.world().get_resource::<p2p::ReadyChannel>()?.subscribe();
    Some(p2p::SwarmChannels {
        commands,
        events,
        ready,
        observed,
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::swarm_channels;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        assert!(swarm_channels(&app).is_none());
        let plugin = discovery::Plugin::new(discovery::Config::default());
        app.add_plugins(plugin);
        app.update();
        assert!(app.world().get_resource::<discovery::Status>().is_some());
    }
}
