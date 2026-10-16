use freenet_stdlib::prelude::ContractKey;

use crate::discovery;
use crate::identity;

pub struct LobbyContract {
    pub client: discovery::freenet::Client,
    pub key: ContractKey,
    pub lobby: discovery::Lobby,
}

#[derive(Clone)]
pub struct FreenetConfig {
    pub host: String,
    pub port: u16,
    pub me: identity::PeerId,
    pub params: Vec<u8>,
    pub room_size: u16,
    pub deploy_policy: discovery::freenet::DeployPolicy,
    pub timing: discovery::Timing,
}
