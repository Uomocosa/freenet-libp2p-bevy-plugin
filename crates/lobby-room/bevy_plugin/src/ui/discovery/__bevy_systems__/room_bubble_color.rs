use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::Color;

#[must_use]
pub const fn room_bubble_color(occupancy: core_discovery::RoomOccupancy) -> Color {
    match occupancy {
        core_discovery::RoomOccupancy::HasSpace => ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR,
        core_discovery::RoomOccupancy::Empty => ui::discovery::ROOM_BUBBLE_EMPTY_COLOR,
        core_discovery::RoomOccupancy::Full => ui::discovery::ROOM_BUBBLE_FULL_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use super::room_bubble_color;
    use crate::discovery as core_discovery;
    use crate::ui;

    #[test]
    fn test_usage() {
        assert_eq!(
            room_bubble_color(core_discovery::RoomOccupancy::Full),
            ui::discovery::ROOM_BUBBLE_FULL_COLOR
        );
    }
}
