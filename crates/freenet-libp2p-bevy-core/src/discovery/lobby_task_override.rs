use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::discovery;
use crate::identity;

#[derive(Resource, Debug, Clone)]
pub struct LobbyTaskOverride {
    pub orchestrator_port: Option<u16>,
    pub peer: identity::Peer,
}

#[atomic_delegates]
impl LobbyTaskOverride {
    pub(crate) fn spawn_fake_lobby_task(
        self,
        channels: discovery::LobbyTaskChannels,
        game: identity::GameId,
    ) -> tokio::task::JoinHandle<()> {
    }
}

#[cfg(test)]
mod tests {
    use super::LobbyTaskOverride;
    use crate::identity;

    #[test]
    fn test_usage() {
        let task_override = LobbyTaskOverride {
            orchestrator_port: Some(4001),
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
        };
        assert_eq!(task_override.orchestrator_port, Some(4001));
    }
}
