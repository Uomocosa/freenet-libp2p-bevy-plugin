use std::collections::BTreeMap;
use std::time::Duration;

use crate::identity;
use crate::spectator;
use crate::time;

#[derive(Default)]
pub struct InZone {
    pub my_zone: Option<identity::ZoneName>,
    pub leases: BTreeMap<identity::PeerId, time::UnixTime>,
    pub last_reported_count: usize,
}

impl InZone {
    pub fn update(
        &mut self,
        input: spectator::InZoneInput,
        now: time::UnixTime,
    ) -> Vec<spectator::InZoneOutput> {
        match input {
            spectator::InZoneInput::MyZone(zone) => self.handle_my_zone(zone),
            spectator::InZoneInput::Lease {
                zone,
                spectator: peer,
                until,
            } => self.handle_lease(&zone, &peer, until, now),
            spectator::InZoneInput::Tick => self.expire_leases(now),
        }
    }

    #[must_use]
    pub fn lease_interval(&self) -> Duration {
        let count = u32::try_from(self.leases.len().saturating_add(1)).unwrap_or(u32::MAX);
        let scaled = spectator::state_machine::LEASE_INTERVAL_PER_SPECTATOR.saturating_mul(count);
        spectator::state_machine::MIN_LEASE_INTERVAL
            .max(scaled)
            .min(spectator::state_machine::MAX_LEASE_INTERVAL)
    }
}

#[rustfmt::skip]
impl InZone {
    #[must_use]
    pub fn spectator_count(&self) -> usize { self.leases.len() }
}

impl InZone {
    pub(crate) fn handle_lease(
        &mut self,
        zone: &identity::ZoneName,
        peer: &identity::PeerId,
        until: time::UnixTime,
        now: time::UnixTime,
    ) -> Vec<spectator::InZoneOutput> {
        if self.my_zone.as_ref() != Some(zone) {
            return Vec::new();
        }
        if until <= now {
            self.leases.remove(peer);
        } else {
            self.leases.insert(peer.clone(), until);
        }
        self.expire_leases(now)
    }
    pub(crate) fn expire_leases(&mut self, now: time::UnixTime) -> Vec<spectator::InZoneOutput> {
        self.leases.retain(|_, until| *until > now);
        self.report_count(self.spectator_count())
    }
    pub(crate) fn handle_my_zone(
        &mut self,
        zone: Option<identity::ZoneName>,
    ) -> Vec<spectator::InZoneOutput> {
        if self.my_zone == zone {
            return Vec::new();
        }
        self.enter_my_zone(zone);
        self.report_count(0)
    }
    pub(crate) fn enter_my_zone(&mut self, zone: Option<identity::ZoneName>) {
        self.my_zone = zone;
        self.leases.clear();
    }
    pub(crate) fn report_count(&mut self, count: usize) -> Vec<spectator::InZoneOutput> {
        if count == self.last_reported_count {
            return Vec::new();
        }
        self.last_reported_count = count;
        vec![spectator::InZoneOutput::SpectatorCountChanged(count)]
    }
}

#[cfg(test)]
mod tests {
    use super::InZone;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut zone = InZone::default();
        zone.update(
            spectator::InZoneInput::MyZone(Some(identity::ZoneName::from("home"))),
            time::UnixTime::EPOCH,
        );
        let outputs = zone.update(
            spectator::InZoneInput::Lease {
                zone: identity::ZoneName::from("home"),
                spectator: identity::PeerId::from("a"),
                until: time::UnixTime::from_secs(10),
            },
            time::UnixTime::from_secs(1),
        );
        assert_eq!(
            outputs,
            vec![spectator::InZoneOutput::SpectatorCountChanged(1)]
        );
        assert_eq!(zone.spectator_count(), 1);
    }

    #[test]
    fn a_lease_for_another_zone_is_ignored() {
        let mut zone = InZone {
            my_zone: Some(identity::ZoneName::from("home")),
            ..Default::default()
        };
        let outputs = zone.update(
            spectator::InZoneInput::Lease {
                zone: identity::ZoneName::from("foreign"),
                spectator: identity::PeerId::from("a"),
                until: time::UnixTime::from_secs(10),
            },
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(zone.spectator_count(), 0);
    }

    #[test]
    fn an_expired_lease_is_ignored_and_a_known_one_is_removed() {
        let mut zone = InZone {
            my_zone: Some(identity::ZoneName::from("home")),
            ..Default::default()
        };
        let expired = zone.update(
            spectator::InZoneInput::Lease {
                zone: identity::ZoneName::from("home"),
                spectator: identity::PeerId::from("a"),
                until: time::UnixTime::from_secs(1),
            },
            time::UnixTime::from_secs(1),
        );
        assert_eq!(expired, Vec::new());
        zone.update(
            spectator::InZoneInput::Lease {
                zone: identity::ZoneName::from("home"),
                spectator: identity::PeerId::from("a"),
                until: time::UnixTime::from_secs(10),
            },
            time::UnixTime::from_secs(1),
        );
        let removed = zone.update(
            spectator::InZoneInput::Lease {
                zone: identity::ZoneName::from("home"),
                spectator: identity::PeerId::from("a"),
                until: time::UnixTime::from_secs(2),
            },
            time::UnixTime::from_secs(2),
        );
        assert_eq!(
            removed,
            vec![spectator::InZoneOutput::SpectatorCountChanged(0)]
        );
        assert!(zone.leases.is_empty());
    }

    #[test]
    fn ticks_expire_leases_and_an_unchanged_count_is_not_reported() {
        let mut zone = InZone {
            my_zone: Some(identity::ZoneName::from("home")),
            ..Default::default()
        };
        zone.leases
            .insert(identity::PeerId::from("a"), time::UnixTime::from_secs(10));
        assert_eq!(
            zone.update(spectator::InZoneInput::Tick, time::UnixTime::from_secs(5)),
            vec![spectator::InZoneOutput::SpectatorCountChanged(1)]
        );
        assert_eq!(
            zone.update(spectator::InZoneInput::Tick, time::UnixTime::from_secs(6)),
            Vec::new()
        );
        assert_eq!(
            zone.update(spectator::InZoneInput::Tick, time::UnixTime::from_secs(11)),
            vec![spectator::InZoneOutput::SpectatorCountChanged(0)]
        );
    }

    #[test]
    fn the_same_zone_changes_nothing_and_a_new_zone_resets_the_count() {
        let mut zone = InZone {
            my_zone: Some(identity::ZoneName::from("home")),
            leases: [(identity::PeerId::from("a"), time::UnixTime::from_secs(10))].into(),
            last_reported_count: 1,
        };
        assert_eq!(
            zone.update(
                spectator::InZoneInput::MyZone(Some(identity::ZoneName::from("home"))),
                time::UnixTime::from_secs(1),
            ),
            Vec::new()
        );
        assert_eq!(zone.spectator_count(), 1);
        assert_eq!(
            zone.update(
                spectator::InZoneInput::MyZone(Some(identity::ZoneName::from("new"))),
                time::UnixTime::from_secs(1),
            ),
            vec![spectator::InZoneOutput::SpectatorCountChanged(0)]
        );
        assert!(zone.leases.is_empty());
    }

    #[test]
    fn the_lease_interval_grows_with_the_spectator_count_and_is_capped() {
        let mut zone = InZone::default();
        assert_eq!(zone.lease_interval(), std::time::Duration::from_secs(15));
        for index in 0..300 {
            zone.leases.insert(
                identity::PeerId::from(format!("spectator-{index}")),
                time::UnixTime::from_secs(10_000),
            );
        }
        assert_eq!(
            zone.lease_interval(),
            spectator::state_machine::MAX_LEASE_INTERVAL
        );
    }
}
