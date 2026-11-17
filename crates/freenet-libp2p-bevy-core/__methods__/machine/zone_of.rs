use crate::identity;
use crate::spectator;

pub fn zone_of<Op: Clone>(
    machine: &spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
) -> Option<&identity::ZoneName> {
    machine
        .sessions
        .get(&slot)
        .and_then(|session| session.zone.as_ref())
}

#[cfg(test)]
mod tests {
    use super::zone_of;
    use crate::identity;
    use crate::spectator;

    #[test]
    fn test_usage() {
        let machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(zone_of(&machine, spectator::SpectateSlot::Preview), None);
    }
}
