use crate::discovery as core_discovery;

#[must_use]
pub fn occupancy_tooltip_text(row: &core_discovery::LobbyRow) -> String {
    let players = format!("{} of {} players", row.occupants, row.room_size);
    match row.friends {
        0 => players,
        1 => format!("{players} | 1 friend"),
        friends => format!("{players} | {friends} friends"),
    }
}
