use crate::ui;
use bevy::prelude::*;

use ui::discovery::PeekedRoomFriends;

pub fn reset_peeked_room_friends(mut peeked: ResMut<PeekedRoomFriends>) {
    peeked.set_if_neq(PeekedRoomFriends::default());
}
