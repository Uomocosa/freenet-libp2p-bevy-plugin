use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{MessageWriter, Resource};

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;
use crate::time;

#[derive(Resource)]
pub struct Machine<Op: Clone> {
    pub game: identity::GameId,
    pub me: Option<identity::PeerId>,
    pub sessions:
        BTreeMap<spectator::SpectateSlot, spectator::state_machine::SpectatorState<Op, Vec<u8>>>,
    pub in_zone: spectator::InZone,
    pub topic_subscriptions: BTreeMap<identity::Topic, u32>,
}

impl<Op: Clone> Machine<Op> {
    #[must_use]
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            me: None,
            sessions: BTreeMap::new(),
            in_zone: spectator::InZone::default(),
            topic_subscriptions: BTreeMap::new(),
        }
    }
}

impl<Op: Clone> Machine<Op> {
    pub fn update(
        &mut self,
        input: spectator::MachineInput<Op, Vec<u8>>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
        match input {
            spectator::MachineInput::Command(command) => self.handle_command(command, now),
            spectator::MachineInput::Session(input) => self.handle_session_input(input, now),
            spectator::MachineInput::MyZone(zone) => {
                self.update_in_zone(spectator::InZoneInput::MyZone(zone), now)
            }
            spectator::MachineInput::Lease {
                zone,
                spectator: peer,
                until,
            } => self.update_in_zone(
                spectator::InZoneInput::Lease {
                    zone,
                    spectator: peer,
                    until,
                },
                now,
            ),
        }
    }
}

#[atomic_delegates]
impl<Op: Clone> Machine<Op> {
    #[must_use]
    pub fn is_lease_owner(&self, slot: spectator::SpectateSlot, zone: &identity::ZoneName) -> bool {
    }
    #[must_use]
    pub fn does_any_slot_watch(&self, zone: &identity::ZoneName) -> bool {}
    #[must_use]
    pub fn zone_of(&self, slot: spectator::SpectateSlot) -> Option<&identity::ZoneName> {}
}

impl<Op: Clone> Machine<Op> {
    pub(crate) fn handle_command(
        &mut self,
        command: spectator::state_machine::SpectateCommand,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
        match command {
            spectator::state_machine::SpectateCommand::Start {
                slot,
                zone,
                history_prefix,
            } => self.start_slot(slot, &zone, history_prefix, now),
            spectator::state_machine::SpectateCommand::Stop { slot } => self.stop_slot(slot, now),
        }
    }

    pub(crate) fn handle_session_input(
        &mut self,
        input: spectator::state_machine::SpectateInput<Op, Vec<u8>>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
        match input {
            spectator::state_machine::SpectateInput::Snapshot {
                history_prefix,
                snapshot,
            } => self.route_snapshot(&history_prefix, snapshot, now),
            spectator::state_machine::SpectateInput::Topic(message) => {
                self.route_topic(message, now)
            }
            spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
                from,
                zone,
                local,
                remote,
            } => self.route_remote_slice_content_hashes(&from, &zone, &local, &remote, now),
            spectator::state_machine::SpectateInput::SliceReceived {
                zone,
                index,
                snapshot,
            } => self.route_slice_received(&zone, index, snapshot, now),
            spectator::state_machine::SpectateInput::Players(players) => {
                self.update_players(players, now)
            }
            spectator::state_machine::SpectateInput::LocalPeer(peer) => {
                self.update_local_peer(peer, now)
            }
            spectator::state_machine::SpectateInput::Tick => self.tick_sessions(now),
        }
    }

    pub(crate) fn update_in_zone(
        &mut self,
        input: spectator::InZoneInput,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
        let outputs = self.in_zone.update(input, now);
        outputs
            .into_iter()
            .map(|output| match output {
                spectator::InZoneOutput::SpectatorCountChanged(count) => {
                    spectator::MachineOutput::SpectatorCountChanged(count)
                }
            })
            .collect()
    }

    pub(crate) fn note_topic_subscription(&mut self, topic: &identity::Topic) -> bool {
        let count = self.topic_subscriptions.entry(topic.clone()).or_insert(0);
        *count = count.saturating_add(1);
        *count == 1
    }
}

#[atomic_delegates]
impl<Op: Clone> Machine<Op> {
    pub(crate) fn carry_out_network_output<S: replica_sync::traits::ReplicatedState<Op = Op>>(
        &self,
        slot: spectator::SpectateSlot,
        output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
        game: &spectator::SpectatorGame,
        network: &mut MessageWriter<p2p::Libp2pCommand>,
    ) {
    }
    pub(crate) fn carry_out_state_output<S: replica_sync::traits::ReplicatedState<Op = Op>>(
        output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
        spectated: Option<&mut spectator::Spectated<S>>,
    ) {
    }
    pub(crate) fn forward_outputs(
        &mut self,
        slot: spectator::SpectateSlot,
        outputs: Vec<spectator::state_machine::SpectateOutput<Op, Vec<u8>>>,
        is_stopping: bool,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn is_output_wanted(
        &mut self,
        slot: spectator::SpectateSlot,
        output: &spectator::state_machine::SpectateOutput<Op, Vec<u8>>,
        is_stopping: bool,
    ) -> bool {
    }
    pub(crate) fn release_topic_subscription(&mut self, topic: &identity::Topic) -> bool {}
    pub(crate) fn route_snapshot(
        &mut self,
        history_prefix: &p2p::HistoryKeyPrefix,
        snapshot: Vec<u8>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn route_topic(
        &mut self,
        message: replica_sync::ReplicaSyncTopicMessage<Op>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn route_remote_slice_content_hashes(
        &mut self,
        from: &identity::PeerId,
        zone: &identity::ZoneName,
        local: &replica_sync::ReplicaSliceContentHashes,
        remote: &replica_sync::ReplicaSliceContentHashes,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn route_slice_received(
        &mut self,
        zone: &identity::ZoneName,
        index: replica_sync::ReplicaSliceIndex,
        snapshot: Vec<u8>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn start_slot(
        &mut self,
        slot: spectator::SpectateSlot,
        zone: &identity::ZoneName,
        history_prefix: p2p::HistoryKeyPrefix,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn stop_slot(
        &mut self,
        slot: spectator::SpectateSlot,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn tick_sessions(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn update_local_peer(
        &mut self,
        peer: Option<identity::PeerId>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn update_players(
        &mut self,
        players: Vec<identity::PeerId>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn update_session(
        &mut self,
        slot: spectator::SpectateSlot,
        input: spectator::state_machine::SpectateInput<Op, Vec<u8>>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
    pub(crate) fn update_sessions_on_zone(
        &mut self,
        zone: &identity::ZoneName,
        input: spectator::state_machine::SpectateInput<Op, Vec<u8>>,
        now: time::UnixTime,
    ) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    }
}

// no test_usage necessary
