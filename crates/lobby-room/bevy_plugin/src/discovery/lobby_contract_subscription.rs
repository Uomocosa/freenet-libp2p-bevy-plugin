use atomic_delegate_macros::atomic_delegates;
use freenet_stdlib::prelude::ContractKey;

use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::time;

pub struct LobbyContractSubscription {
    pub client: freenet::Client,
    pub key: ContractKey,
    pub lobby: discovery::Lobby,
    pub(crate) has_unread_subscribe_answer: bool,
}

#[atomic_delegates]
impl LobbyContractSubscription {
    pub fn resubscribe_to_heal_replica_split(&self) -> Result<(), discovery::Error> {}
    pub fn publish_heartbeat(
        &self,
        room: &identity::RoomName,
        me: &identity::PeerId,
        room_member: discovery::RoomMember,
        room_size: discovery::RoomSize,
        created_at: time::UnixTime,
    ) -> Result<(), discovery::Error> {
    }
    pub fn take_subscribe_answer(&mut self) -> bool {}
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
