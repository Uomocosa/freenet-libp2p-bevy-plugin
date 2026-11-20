use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq)]
pub enum SyntheticInput {
    Glide {
        from: Vec2,
        to: Vec2,
        speed: testing::PointerSpeed,
    },
    Button(MouseButton, ButtonState),
    Keys(Vec<testing::KeyEvent>),
    Wait(testing::Millis),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStep {
    Text(String),
    Tap(testing::Key),
    Down(testing::Key),
    Up(testing::Key),
    Chord(Vec<testing::Key>),
    Repeat {
        key: testing::Key,
        count: testing::RepeatCount,
    },
    Wait(testing::Millis),
    SameFrame(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerStep {
    Moved(Vec2),
    Button(ButtonState),
}

#[derive(Error, Debug)]
pub enum KeyStepError {
    #[error("a Wait step cannot sit inside SameFrame: a frame group holds key events only")]
    WaitInSameFrame,
}
