use libp2p::StreamProtocol;

use crate::identity;

pub fn sync_protocol(game: &identity::GameId) -> Result<StreamProtocol, String> {
    libp2p::StreamProtocol::try_from_owned(format!("/{}/replica-sync/2", game.as_str()))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::sync_protocol;
    use crate::identity;

    #[test]
    fn test_usage() {
        let protocol = sync_protocol(&identity::GameId::from("game")).expect("valid protocol");
        assert_eq!(protocol.as_ref(), "/game/replica-sync/2");
    }
}
