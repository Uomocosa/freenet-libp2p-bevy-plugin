use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::discovery;

pub struct LobbyStepView<'a> {
    pub status: &'a discovery::state_machine::DiscoveryState,
    pub join_error: Option<&'a str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LobbyStatus {
    pub rooms: Vec<String>,
    pub joined_room: Option<String>,
    pub other_members: usize,
    pub connected: usize,
    pub lobby_fetched: bool,
    pub join_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocatedNode {
    pub centre: Vec2,
    pub disabled: bool,
}

// no test_usage necessary
