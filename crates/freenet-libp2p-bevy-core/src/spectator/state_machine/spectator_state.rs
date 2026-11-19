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
    pub me: Option<identity::PeerId>,
    pub zone: Option<identity::ZoneName>,
    pub history_prefix: Option<p2p::HistoryKeyPrefix>,
    pub phase: spectator::state_machine::SpectatePhase,
    pub buffered_ops: Vec<(replica_sync::OpMeta, Op)>,
    pub applied_metas: Vec<replica_sync::OpMeta>,
    pub players: BTreeSet<identity::PeerId>,
    pub next_lease_at: time::UnixTime,
    pub last_slice_request_at: BTreeMap<identity::PeerId, time::UnixTime>,
    pub pending_slice_requests: BTreeMap<replica_sync::ReplicaSliceIndex, time::UnixTime>,
    pub _snap: PhantomData<Snap>,
}

impl<Op, Snap> SpectatorState<Op, Snap> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            me: None,
            zone: None,
            history_prefix: None,
            phase: spectator::state_machine::SpectatePhase::Idle,
            buffered_ops: Vec::new(),
            applied_metas: Vec::new(),
            players: BTreeSet::new(),
            next_lease_at: time::UnixTime::EPOCH,
            last_slice_request_at: BTreeMap::new(),
            pending_slice_requests: BTreeMap::new(),
            _snap: PhantomData,
        }
    }
}

impl<Op, Snap> SpectatorState<Op, Snap> {
    pub fn update(
        &mut self,
        input: spectator::state_machine::SpectateInput<Op, Snap>,
        now: time::UnixTime,
        lease_interval: Duration,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        match input {
            spectator::state_machine::SpectateInput::Snapshot { snapshot, .. } => {
                self.handle_snapshot(snapshot)
            }
            spectator::state_machine::SpectateInput::Topic(message) => self.handle_topic(message),
            spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
                from,
                zone,
                local,
                remote,
            } => self.handle_remote_slice_content_hashes(&from, &zone, &local, &remote, now),
            spectator::state_machine::SpectateInput::SliceReceived {
                index, snapshot, ..
            } => self.handle_slice_received(index, snapshot),
            spectator::state_machine::SpectateInput::Players(players) => {
                self.handle_players(players)
            }
            spectator::state_machine::SpectateInput::LocalPeer(peer) => {
                self.handle_local_peer(peer, now, lease_interval)
            }
            spectator::state_machine::SpectateInput::Tick => self.handle_tick(now, lease_interval),
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
        &mut self,
        index: replica_sync::ReplicaSliceIndex,
        snapshot: Snap,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
        self.pending_slice_requests.remove(&index);
        if self.phase != spectator::state_machine::SpectatePhase::Live {
            return Vec::new();
        }
        vec![spectator::state_machine::SpectateOutput::Merge(snapshot)]
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
}

#[atomic_delegates]
impl<Op, Snap> SpectatorState<Op, Snap> {
    pub(crate) fn has_seen_meta(&self, meta: replica_sync::OpMeta) -> bool {}
    pub(crate) fn remember_applied_meta(&mut self, meta: replica_sync::OpMeta) {}
    pub(crate) fn expire_slice_requests(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_local_peer(
        &mut self,
        peer: Option<identity::PeerId>,
        now: time::UnixTime,
        lease_interval: Duration,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_remote_op(
        &mut self,
        zone: &identity::ZoneName,
        meta: replica_sync::OpMeta,
        op: Op,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn handle_tick(
        &mut self,
        now: time::UnixTime,
        lease_interval: Duration,
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
        lease_interval: Duration,
    ) -> Vec<spectator::state_machine::SpectateOutput<Op, Snap>> {
    }
    pub(crate) fn start_spectating(
        &mut self,
        zone: &identity::ZoneName,
        history_prefix: p2p::HistoryKeyPrefix,
        now: time::UnixTime,
        lease_interval: Duration,
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
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = spectator::state_machine::SpectatorState::<u8, Vec<u8>>::new(
            identity::GameId::from("game"),
        );
        let outputs = state.update(
            spectator::state_machine::SpectateInput::Tick,
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Idle);
    }

    #[test]
    fn a_lease_is_republished_every_half_interval() {
        let mut state = testing::spectating_state(
            Some("me"),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let half = time::UnixTime::from_millis(7_500);
        assert_eq!(
            testing::lease_outputs(&state.update(
                spectator::state_machine::SpectateInput::Tick,
                time::UnixTime::from_millis(7_499),
                spectator::state_machine::MIN_LEASE_INTERVAL,
            )),
            Vec::new()
        );
        assert_eq!(
            testing::lease_outputs(&state.update(
                spectator::state_machine::SpectateInput::Tick,
                half,
                spectator::state_machine::MIN_LEASE_INTERVAL,
            )),
            vec![half.saturating_add(std::time::Duration::from_secs(45))]
        );
    }

    #[test]
    fn stop_publishes_a_lease_that_has_already_expired() {
        let mut state = testing::spectating_state(
            Some("me"),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let now = time::UnixTime::from_secs(30);
        let outputs = state.stop_spectating(now);
        assert_eq!(testing::lease_outputs(&outputs), vec![now]);
        assert_eq!(state.phase, spectator::state_machine::SpectatePhase::Idle);
    }

    #[test]
    fn a_received_slice_frees_its_pending_request() {
        let mut state = testing::spectating_state(
            Some("me"),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        state.phase = spectator::state_machine::SpectatePhase::Live;
        let index = replica_sync::ReplicaSliceIndex(1);
        state
            .pending_slice_requests
            .insert(index, time::UnixTime::EPOCH);
        state.update(
            spectator::state_machine::SpectateInput::SliceReceived {
                zone: identity::ZoneName::from("zone"),
                index,
                snapshot: Vec::new(),
            },
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert!(state.pending_slice_requests.is_empty());
    }
}
