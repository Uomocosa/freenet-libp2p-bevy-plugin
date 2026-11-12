use bevy::prelude::SystemSet;

/// Systems whose output depends on the wall clock; tests switch them off by default.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WallClockSystem {
    RoomBrushes,
}

#[cfg(test)]
impl WallClockSystem {
    pub const ALL: [Self; 1] = [Self::RoomBrushes];
}
