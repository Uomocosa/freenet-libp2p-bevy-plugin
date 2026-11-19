use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeWireHello {
    pub peer: identity::Peer,
    pub game: identity::GameId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerPath {
    pub button: MouseButton,
    pub points: Vec<Vec2>,
    pub speed: testing::PointerSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CursorTarget {
    pub x: f32,
    pub y: f32,
    pub speed: testing::PointerSpeed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectDialBookEntry {
    pub peer_id: String,
    pub kind: direct_dial::KnownPeerKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectDialStatus {
    pub peer_id: String,
    pub listen_addrs: Vec<String>,
    pub connected: Vec<String>,
    pub book: Vec<DirectDialBookEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: testing::Key,
    pub state: ButtonState,
    pub does_repeat: bool,
    pub does_emit_text: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyValueOp {
    pub key: u8,
    pub value: u32,
}
