use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpectateSlot {
    Preview,
    Stream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectateFreshness {
    #[default]
    Bootstrapping,
    Live,
    Repairing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapOutcome {
    RequestChunks(Vec<p2p::ZoneHistoryRecordKind>),
    Snapshot {
        history_prefix: p2p::HistoryKeyPrefix,
        snapshot: Vec<u8>,
    },
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InZoneInput {
    MyZone(Option<identity::ZoneName>),
    Lease {
        zone: identity::ZoneName,
        spectator: identity::PeerId,
        until: time::UnixTime,
    },
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InZoneOutput {
    SpectatorCountChanged(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineInput<Op, Snap> {
    Command(spectator::state_machine::SpectateCommand),
    Session(spectator::state_machine::SpectateInput<Op, Snap>),
    MyZone(Option<identity::ZoneName>),
    Lease {
        zone: identity::ZoneName,
        spectator: identity::PeerId,
        until: time::UnixTime,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineOutput<Op, Snap> {
    Session {
        slot: spectator::SpectateSlot,
        output: spectator::state_machine::SpectateOutput<Op, Snap>,
    },
    SpectatorCountChanged(usize),
}

// no test_usage necessary
