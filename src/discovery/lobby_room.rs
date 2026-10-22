use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct LobbyRoom {
    pub room_size: discovery::RoomSize,
    pub created_at: discovery::UnixTime,
    pub members: BTreeMap<identity::PeerId, discovery::RoomMember>,
}

impl LobbyRoom {
    #[must_use]
    pub fn merge_room(current: Option<Self>, incoming: Self) -> Self {
        match current {
            Some(mut existing) if existing.created_at == incoming.created_at => {
                for (peer, room_member) in incoming.members {
                    let merged = merge_room_member(existing.members.remove(&peer), room_member);
                    existing.members.insert(peer, merged);
                }
                existing.room_size = existing.room_size.max(incoming.room_size);
                existing
            }
            Some(existing) if existing.created_at > incoming.created_at => existing,
            _ => incoming,
        }
    }
}

// needed helper: keeps the newer of two member records
fn merge_room_member(
    current: Option<discovery::RoomMember>,
    incoming: discovery::RoomMember,
) -> discovery::RoomMember {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

// no test_usage necessary
