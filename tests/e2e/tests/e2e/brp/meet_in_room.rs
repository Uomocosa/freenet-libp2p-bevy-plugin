use std::path::Path;

use super::{BrpNetwork, LobbyRoomPeer, PEER_WINDOWS};

/// Launches `peers` apps, has `p1` create `room`, has the others join it, and waits until every
/// peer lists `peers - 1` other members. Returns the peers and the last button press (day millis),
/// the moment the old `lobby_room_join` measured its budget from.
///
/// # Errors
/// Returns an error if a peer does not start, the room is not created or joined, or the room does
/// not reach `peers` members.
pub async fn meet_in_room(
    bin: &Path,
    net: &BrpNetwork,
    peers: usize,
    room: &str,
) -> Result<(Vec<LobbyRoomPeer>, u64), String> {
    let mut members = Vec::with_capacity(peers);
    for index in 0..peers {
        let name = format!("p{}", index.saturating_add(1));
        let window = PEER_WINDOWS
            .iter()
            .cycle()
            .nth(index)
            .copied()
            .unwrap_or((0, 0));
        members.push(LobbyRoomPeer::launch(bin, &name, net, window).await?);
    }
    let Some((creator, joiners)) = members.split_first() else {
        return Err("meet_in_room needs at least one peer".to_string());
    };
    let mut last_press = creator.create_room_timed(room).await?;
    for joiner in joiners {
        last_press = last_press.max(joiner.join_room_timed(room).await?);
    }
    let others = peers.saturating_sub(1);
    for member in &members {
        member
            .wait_until_room_has_at_least_other_members(others)
            .await?;
    }
    Ok((members, last_press))
}
