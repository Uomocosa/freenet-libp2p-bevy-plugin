use freenet_stdlib::prelude::ContractKey;

use crate::discovery;
use crate::identity;

pub struct LobbyContractSubscription {
    pub client: discovery::freenet::Client,
    pub key: ContractKey,
    pub lobby: discovery::Lobby,
}

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub me: identity::PeerId,
    pub params: Vec<u8>,
    pub discovery: discovery::Config,
}
