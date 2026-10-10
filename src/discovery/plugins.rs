use bevy::app::{PluginGroup, PluginGroupBuilder};
use derive_more::Deref;

use crate::discovery;

#[derive(Deref)]
pub struct Plugins(pub discovery::Config);

impl PluginGroup for Plugins {
    fn build(self) -> PluginGroupBuilder {
        let Self(config) = self;
        let group = PluginGroupBuilder::start::<Self>().add(discovery::Plugin::new(config));
        #[cfg(feature = "default_ui")]
        let group = group.add(discovery::ui::DefaultUiPlugin);
        group
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugins;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(Plugins(discovery::Config::default()));
        assert!(app.world().get_resource::<discovery::Snapshot>().is_some());
    }
}
