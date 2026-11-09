use serde::{Deserialize, Serialize};

use crate::Peer;
use crate::RoomMembership;
use crate::UnixTime;
use crate::Username;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct RoomMember {
    pub peer: Peer,
    pub display_name: Option<Username>,
    pub updated_at: UnixTime,
    pub membership: RoomMembership,
}
