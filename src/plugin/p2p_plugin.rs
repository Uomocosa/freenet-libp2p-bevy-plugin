use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::p2p;
use crate::plugin;

#[derive(Deref)]
pub struct P2PPlugin<T: p2p::Message>(pub plugin::Config<T>);

#[atomic_delegates]
impl<T: p2p::Message> P2PPlugin<T> {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[rustfmt::skip]
impl<T: p2p::Message> bevy::prelude::Plugin for P2PPlugin<T> {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::P2PPlugin;
    use crate::net_id;
    use crate::p2p;
    use crate::plugin;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(P2PPlugin(plugin::Config::<Dummy>::new(
            net_id::NetworkId(1),
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )));
        app.update();
        assert!(app.world().get_resource::<p2p::Events<Dummy>>().is_some());
        assert!(app.world().get_resource::<p2p::Outbox>().is_some());
        assert!(app.world().get_resource::<p2p::EventTap>().is_some());
        assert!(app.world().get_resource::<p2p::Signals>().is_some());
    }
}
