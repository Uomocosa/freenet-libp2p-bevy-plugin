use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;
use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FakeWireClientFrame {
    Libp2pCommand(p2p::Libp2pCommand),
    JoinedZoneBytes(Vec<u8>),
    SpecificZoneBytes {
        zone: identity::ZoneName,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyntheticInput {
    Glide {
        from: Vec2,
        to: Vec2,
        speed: testing::PointerSpeed,
    },
    Button(MouseButton, ButtonState),
    Key(char, ButtonState),
    Named(testing::Key, ButtonState),
}
