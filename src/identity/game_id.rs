use derive_more::{Deref, Display, From};
use libp2p::StreamProtocol;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, From)]
#[from(forward)]
pub struct GameId(pub String);

impl GameId {
    #[must_use]
    pub fn new_game_id() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let digest = blake3::hash(format!("{nanos}-{}", std::process::id()).as_bytes());
        Self(digest.to_hex().as_str().chars().take(16).collect())
    }
}

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
