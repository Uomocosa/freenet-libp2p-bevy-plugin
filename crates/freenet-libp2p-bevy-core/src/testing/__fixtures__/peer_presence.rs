use std::collections::BTreeMap;

use crate::identity;
use crate::presence;

#[must_use]
pub fn peer_presence(
    id: &str,
    display_name: Option<&str>,
    zone: identity::Zone,
) -> presence::PeerPresence {
    presence::PeerPresence(BTreeMap::from([(
        identity::PeerId::from(id),
        presence::Whereabouts {
            zone,
            display_name: display_name.map(identity::Username::from),
        },
    )]))
}

#[cfg(test)]
mod tests {
    use crate::identity;

    use super::peer_presence;

    #[test]
    fn test_usage() {
        let observed = peer_presence("bob", Some("Bob"), identity::Zone::Global);
        let bob = observed
            .get(&identity::PeerId::from("bob"))
            .expect("bob is present");
        assert_eq!(bob.zone, identity::Zone::Global);
        assert_eq!(
            bob.display_name
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("Bob")
        );
    }
}
