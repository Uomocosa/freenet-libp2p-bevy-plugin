use crate::discovery;

pub fn merge_cached_lobby(
    client: &mut discovery::freenet::LobbyClient,
    incoming: discovery::Lobby,
) {
    let cached = std::mem::take(&mut client.lobby);
    client.lobby = discovery::freenet::merge_lobby(cached, incoming);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::merge_cached_lobby);
    }
}
