use atomic_delegate_macros::atomic_delegates;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::PeerId;
use crate::RoomMember;
use crate::RoomSize;
use crate::UnixTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct Room {
    pub room_size: RoomSize,
    pub created_at: UnixTime,
    pub members: BTreeMap<PeerId, RoomMember>,
}

impl Room {
    #[must_use]
    pub fn merge_room(current: Option<Self>, incoming: Self) -> Self {
        match current {
            Some(mut existing) if existing.created_at == incoming.created_at => {
                for (peer, room_member) in incoming.members {
                    let merged =
                        Self::merge_room_member(existing.members.remove(&peer), room_member);
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

#[atomic_delegates]
impl Room {
    #[must_use]
    pub(crate) fn newest_room_member(&self) -> UnixTime {}
}

impl Room {
    #[must_use]
    pub(crate) fn merge_room_member(
        current: Option<RoomMember>,
        incoming: RoomMember,
    ) -> RoomMember {
        match current {
            Some(existing) if existing.updated_at >= incoming.updated_at => existing,
            _ => incoming,
        }
    }
}

// no test_usage necessary
