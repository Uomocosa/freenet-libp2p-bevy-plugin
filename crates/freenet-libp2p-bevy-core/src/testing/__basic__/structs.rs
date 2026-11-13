use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::direct_dial;
use crate::identity;
use crate::p2p;
use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedEvent {
    pub to: identity::PeerId,
    pub event: p2p::Libp2pEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanOut {
    pub to: testing::FanOutTarget,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepClock {
    pub elapsed: Duration,
    pub is_first_poll: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E2EStepPoll {
    pub outcome: testing::StepOutcome,
    pub effect: Option<testing::E2EStepEffect>,
}

pub struct E2EStepView {
    pub room_message_senders: usize,
    pub members: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepTiming {
    pub raw: String,
    pub secs: f64,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeWireHello {
    pub peer: identity::Peer,
    pub game: identity::GameId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredStep {
    pub text: String,
    pub is_wait: bool,
    pub within: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug)]
pub struct RunningPeer {
    pub log: PathBuf,
    pub child: Option<std::process::Child>,
    pub exit: Option<testing::PeerExit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookKindExpectation {
    pub owner: String,
    pub peer: String,
    pub kind: direct_dial::KnownPeerKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePeer {
    pub name: String,
    pub peer: identity::Peer,
}
