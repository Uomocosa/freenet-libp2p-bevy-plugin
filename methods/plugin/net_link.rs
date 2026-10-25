use bevy::prelude::App;

use crate::discovery;
use crate::p2p;

pub fn net_link(app: &App) -> Option<discovery::libp2p::NetLink> {
    let events = app
        .world()
        .get_resource::<p2p::NetEventChannel>()?
        .subscribe();
    let commands = (**app.world().get_resource::<p2p::NetCommandSender>()?).clone();
    let (ready, observed) = app.world().get_resource::<p2p::ReadyChannel>()?.subscribe();
    Some(discovery::libp2p::NetLink {
        commands,
        events,
        ready,
        observed,
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::net_link;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        assert!(net_link(&app).is_none());
        let plugin = discovery::Plugin::new(discovery::Config::default());
        app.add_plugins(plugin);
        app.update();
        assert!(app.world().get_resource::<discovery::Status>().is_some());
    }
}
