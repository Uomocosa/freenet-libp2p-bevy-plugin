use crate::identity;

#[must_use]
pub fn from_zone(game: &identity::GameId, zone: &identity::ZoneName) -> identity::Topic {
    let path = format!("{game}/zone/{zone}");
    identity::Topic::from(path)
}

#[cfg(test)]
mod tests {
    use super::from_zone;
    use crate::identity;

    #[test]
    fn test_usage() {
        let topic = from_zone(
            &identity::GameId::from("game/id"),
            &identity::ZoneName::from("zone-a"),
        );
        assert_eq!(topic.as_str(), "game/id/zone/zone-a");
    }
}
