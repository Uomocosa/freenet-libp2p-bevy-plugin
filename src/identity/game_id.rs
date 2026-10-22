use derive_more::{Deref, Display, From};
use libp2p::StreamProtocol;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From)]
#[from(forward)]
pub struct GameId(pub String);

#[rustfmt::skip]
impl GameId {
    #[must_use]
    pub fn contract_params(&self) -> Vec<u8> { self.as_bytes().to_vec() }

    pub fn sync_protocol(&self) -> Result<StreamProtocol, String> {
        libp2p::StreamProtocol::try_from_owned(format!("/{}/room-sync/2", self.as_str()))
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::GameId;

    #[test]
    fn test_usage() {
        assert_eq!(GameId::from("game").as_str(), "game");
    }
}
