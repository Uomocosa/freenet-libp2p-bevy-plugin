use crate::discovery as core_discovery;

#[must_use]
pub fn room_numbers(row: &core_discovery::LobbyRow) -> [u32; 3] {
    [
        u32::try_from(row.occupants).unwrap_or(u32::MAX),
        u32::from(row.room_size.get()),
        u32::try_from(row.friends).unwrap_or(u32::MAX),
    ]
}
