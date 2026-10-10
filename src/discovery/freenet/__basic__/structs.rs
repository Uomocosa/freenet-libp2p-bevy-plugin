use freenet_stdlib::prelude::ContractKey;

use crate::discovery;

pub struct LobbyClient {
    pub client: discovery::freenet::Client,
    pub key: ContractKey,
    pub lobby: discovery::Lobby,
}
