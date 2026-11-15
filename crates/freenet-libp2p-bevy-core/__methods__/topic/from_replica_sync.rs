use crate::identity;

#[must_use]
pub fn from_replica_sync(game: &identity::GameId, zone: &identity::ZoneName) -> identity::Topic {
    let path = format!("{game}/zone/{zone}/sync");
    identity::Topic::from(path)
}

#[cfg(test)]
mod tests {
    use super::from_replica_sync;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = from_replica_sync(
            &identity::GameId::from("game"),
            &identity::ZoneName::from("zone-a"),
        );
        assert_eq!(topic.as_str(), "game/zone/zone-a/sync");
    }
}
