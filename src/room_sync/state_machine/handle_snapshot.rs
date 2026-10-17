use crate::identity;
use crate::room_sync;

pub fn handle_snapshot<O>(
    state: &mut room_sync::state_machine::State<O>,
    room: identity::RoomName,
    part: u32,
    total: u32,
    data: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<O>> {
    if state.phase != room_sync::state_machine::Phase::Fetching
        || state.room.as_ref() != Some(&room)
    {
        return Vec::new();
    }
    let Ok(total) = usize::try_from(total) else {
        return Vec::new();
    };
    let Ok(index) = usize::try_from(part) else {
        return Vec::new();
    };
    if state.parts.len() != total {
        state.total = u32::try_from(total).unwrap_or(u32::MAX);
        state.parts = vec![None; total];
    }
    let Some(slot) = state.parts.get_mut(index) else {
        return Vec::new();
    };
    *slot = Some(data);
    if let Some(next) = state.parts.iter().position(Option::is_none) {
        let Some(peer) = state.source.clone() else {
            return Vec::new();
        };
        return vec![room_sync::state_machine::Output::Fetch {
            peer,
            room,
            part: u32::try_from(next).unwrap_or(u32::MAX),
        }];
    }
    let complete: Vec<Vec<u8>> = state.parts.iter().flatten().cloned().collect();
    let bytes = room_sync::join_parts(&complete);
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = vec![room_sync::state_machine::Output::Merge { bytes }];
    for (meta, op) in state.buffered.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

// no test_usage necessary
