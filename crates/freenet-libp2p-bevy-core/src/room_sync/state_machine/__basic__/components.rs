use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoomLiveness {
    pub is_room_still_listed: bool,
    pub has_other_history_holders: bool,
}
