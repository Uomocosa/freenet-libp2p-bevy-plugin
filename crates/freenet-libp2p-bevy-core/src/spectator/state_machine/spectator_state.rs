use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub struct SpectatorState<Op, Snap> {
    pub game: identity::GameId,
    pub me: identity::PeerId,
    pub zone: Option<identity::ZoneName>,
    pub phase: spectator::state_machine::SpectatePhase,
    pub buffered_ops: Vec<(replica_sync::OpMeta, Op)>,
    pub applied_metas: Vec<replica_sync::OpMeta>,
    pub players: BTreeSet<identity::PeerId>,
    pub leases: BTreeMap<identity::PeerId, time::UnixTime>,
    pub next_lease_at: time::UnixTime,
    pub last_reported_count: usize,
    pub last_slice_request_at: BTreeMap<identity::PeerId, time::UnixTime>,
    pub _snap: PhantomData<Snap>,
}

impl<Op, Snap> SpectatorState<Op, Snap> {
    #[must_use]
    pub const fn new(game: identity::GameId, me: identity::PeerId) -> Self {
        Self {
            game,
            me,
            zone: None,
            phase: spectator::state_machine::SpectatePhase::Idle,
            buffered_ops: Vec::new(),
            applied_metas: Vec::new(),
            players: BTreeSet::new(),
            leases: BTreeMap::new(),
            next_lease_at: time::UnixTime::EPOCH,
            last_reported_count: 0,
            last_slice_request_at: BTreeMap::new(),
            _snap: PhantomData,
        }
    }
}

impl<Op, Snap> SpectatorState<Op, Snap> {
    pub fn update(
        &mut self,
        input: spectator::state_machine::SpectateInput<Op, Snap>,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        match input {
            spectator::state_machine::SpectateInput::Command(command) => {
                self.handle_command(command, now)
            }
            spectator::state_machine::SpectateInput::Snapshot(snapshot) => {
                self.handle_snapshot(snapshot)
            }
            spectator::state_machine::SpectateInput::Topic(message) => self.handle_topic(message),
            spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
                from,
                zone,
                local,
                remote,
            } => self.handle_remote_slice_content_hashes(&from, &zone, &local, &remote, now),
            spectator::state_machine::SpectateInput::SliceReceived { snapshot, .. } => {
                self.handle_slice_received(snapshot)
            }
            spectator::state_machine::SpectateInput::Players(players) => {
                self.handle_players(players)
            }
            spectator::state_machine::SpectateInput::Lease {
                zone,
                spectator,
                until,
            } => self.handle_lease(&zone, &spectator, until, now),
            spectator::state_machine::SpectateInput::Tick => self.handle_tick(now),
        }
    }

    pub(crate) fn handle_command(
        &mut self,
        command: spectator::state_machine::SpectateCommand,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        match command {
            spectator::state_machine::SpectateCommand::Start {
                zone,
                history_prefix,
            } => self.start_spectating(&zone, history_prefix, now),
            spectator::state_machine::SpectateCommand::Stop => self.stop_spectating(now),
        }
    }

    pub(crate) fn handle_players(
        &mut self,
        players: Vec<identity::PeerId>,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        self.players = players.into_iter().collect();
        Vec::new()
    }

    pub(crate) fn handle_slice_received(
        &self,
        snapshot: Snap,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        if self.phase != spectator::state_machine::SpectatePhase::Live {
            return Vec::new();
        }
        vec![spectator::state_machine::SpectateOutput::Merge(snapshot)]
    }

    pub(crate) fn handle_tick(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        let mut outputs = self.expire_leases(now);
        if self.zone.is_some() && now >= self.next_lease_at {
            outputs.extend(self.publish_lease(now));
        }
        outputs
    }

    pub(crate) fn handle_topic(
        &mut self,
        message: replica_sync::ReplicaSyncTopicMessage<Op>,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        match message {
            replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { zone, meta, op } => {
                self.handle_remote_op(&zone, meta, op)
            }
            replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes { .. }
            | replica_sync::ReplicaSyncTopicMessage::SpectatorLease { .. } => Vec::new(),
        }
    }

    pub(crate) fn lease_interval(&self) -> Duration {
        let count = u32::try_from(self.leases.len().saturating_add(1)).unwrap_or(u32::MAX);
        let scaled = spectator::state_machine::LEASE_INTERVAL_PER_SPECTATOR.saturating_mul(count);
        spectator::state_machine::MIN_LEASE_INTERVAL
            .max(scaled)
            .min(spectator::state_machine::MAX_LEASE_INTERVAL)
    }
}

#[rustfmt::skip]
impl<Op, Snap> SpectatorState<Op, Snap> {
    pub(crate) fn next_lease_after(&self, now: time::UnixTime) -> time::UnixTime { now.saturating_add(self.lease_interval().checked_div(2).unwrap_or_default()) }
}

#[atomic_delegates]
impl<Op, Snap> SpectatorState<Op, Snap> {
    pub(crate) fn has_seen_meta(&self, meta: replica_sync::OpMeta) -> bool {}
    pub(crate) fn lease_until(&self, now: time::UnixTime) -> time::UnixTime {}
    pub(crate) fn remember_applied_meta(&mut self, meta: replica_sync::OpMeta) {}
    pub(crate) fn expire_leases(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_lease(
        &mut self,
        zone: &identity::ZoneName,
        spectator: &identity::PeerId,
        until: time::UnixTime,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_remote_op(
        &mut self,
        zone: &identity::ZoneName,
        meta: replica_sync::OpMeta,
        op: Op,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_remote_slice_content_hashes(
        &mut self,
        from: &identity::PeerId,
        zone: &identity::ZoneName,
        local: &replica_sync::ReplicaSliceContentHashes,
        remote: &replica_sync::ReplicaSliceContentHashes,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_snapshot(
        &mut self,
        snapshot: Snap,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn publish_lease(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn start_spectating(
        &mut self,
        zone: &identity::ZoneName,
        history_prefix: p2p::HistoryKeyPrefix,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn stop_spectating(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        let outputs = state.update(
            spectator::state_machine::SpectateInput::Tick,
            time::UnixTime::EPOCH,
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Idle);
    }

    fn spectating_state() -> spectator::state_machine::SpectatorState<u8, Vec<u8>> {
        let mut state = spectator::state_machine::SpectatorState::new(
            identity::GameId::from("game"),
            identity::PeerId::from("me"),
        );
        state.update(
            spectator::state_machine::SpectateInput::Command(
                spectator::state_machine::SpectateCommand::Start {
                    zone: identity::ZoneName::from("zone"),
                    history_prefix: p2p::HistoryKeyPrefix::from("prefix"),
                },
            ),
            time::UnixTime::EPOCH,
        );
        state
    }

    fn lease_outputs(
        outputs: &[spectator::state_machine::SpectateOutput<u8, Vec<u8>>],
    ) -> Vec<time::UnixTime> {
        outputs
            .iter()
            .filter_map(|output| match output {
                spectator::state_machine::SpectateOutput::PublishLease { until, .. } => {
                    Some(*until)
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_lease_is_republished_every_half_interval() {
        let mut state = spectating_state();
        let half = time::UnixTime::from_millis(7_500);
        assert_eq!(
            lease_outputs(&state.update(
                spectator::state_machine::SpectateInput::Tick,
                time::UnixTime::from_millis(7_499),
            )),
            Vec::new()
        );
        assert_eq!(
            lease_outputs(&state.update(spectator::state_machine::SpectateInput::Tick, half,)),
            vec![half.saturating_add(std::time::Duration::from_secs(45))]
        );
    }

    #[test]
    fn the_lease_interval_grows_with_the_spectator_count_and_is_capped() {
        let mut state = spectating_state();
        assert_eq!(state.lease_interval(), std::time::Duration::from_secs(15));
        for index in 0..99 {
            state.leases.insert(
                identity::PeerId::from(format!("spectator-{index}")),
                time::UnixTime::from_secs(10_000),
            );
        }
        assert_eq!(state.lease_interval(), std::time::Duration::from_secs(50));
        for index in 0..300 {
            state.leases.insert(
                identity::PeerId::from(format!("more-{index}")),
                time::UnixTime::from_secs(10_000),
            );
        }
        assert_eq!(
            state.lease_interval(),
            spectator::state_machine::MAX_LEASE_INTERVAL
        );
    }

    #[test]
    fn stop_publishes_a_lease_that_has_already_expired() {
        let mut state = spectating_state();
        let now = time::UnixTime::from_secs(30);
        let outputs = state.update(
            spectator::state_machine::SpectateInput::Command(
                spectator::state_machine::SpectateCommand::Stop,
            ),
            now,
        );
        assert_eq!(lease_outputs(&outputs), vec![now]);
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Idle);
    }
}
