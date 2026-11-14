use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Pending,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum E2EStepEffect {
    ArmRoomAction(testing::Action),
    Extension(Vec<u8>),
    AbortProcess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FakeWireClientFrame {
    Libp2pCommand(p2p::Libp2pCommand),
    JoinedRoomBytes(Vec<u8>),
    SpecificRoomBytes {
        room: identity::RoomName,
        data: Vec<u8>,
    },
    Extension(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FakeWireServerFrame {
    Libp2pEvent(p2p::Libp2pEvent),
    Extension(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanOutTarget {
    Peer(identity::PeerId),
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerExit {
    Code(i32),
    Signal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyntheticInput {
    Glide {
        from: Vec2,
        to: Vec2,
        speed: testing::PointerSpeed,
    },
    Button(MouseButton, ButtonState),
    Key(char, ButtonState),
}
