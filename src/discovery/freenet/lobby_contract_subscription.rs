use atomic_delegate_macros::atomic_delegates;
use freenet_stdlib::prelude::ContractKey;

use crate::discovery;
use crate::identity;

pub struct LobbyContractSubscription {
    pub client: discovery::freenet::Client,
    pub key: ContractKey,
    pub lobby: discovery::Lobby,
}

#[atomic_delegates]
impl LobbyContractSubscription {
    pub fn bridge_subscription(&mut self) -> Result<(), discovery::Error> {}
    pub fn publish_heartbeat(
        &mut self,
        room: &identity::RoomName,
        me: &identity::PeerId,
        room_member: discovery::RoomMember,
        room_size: discovery::RoomSize,
        created_at: discovery::UnixTime,
    ) -> Result<(), discovery::Error> {
    }
    pub async fn poll(&mut self) -> Result<discovery::Lobby, discovery::Error> {}
    pub async fn refresh(&mut self) -> Result<discovery::Lobby, discovery::Error> {}
}

impl LobbyContractSubscription {
    pub(crate) fn absorb_bytes(&mut self, bytes: &[u8]) -> Result<(), discovery::Error> {
        let incoming: discovery::Lobby = bincode::deserialize(bytes)?;
        self.lobby.merge(incoming);
        Ok(())
    }
}

// no test_usage necessary
