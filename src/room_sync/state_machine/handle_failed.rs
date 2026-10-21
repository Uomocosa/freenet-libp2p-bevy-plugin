use crate::identity;
use crate::room_sync;

pub fn handle_failed<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    from: &identity::PeerId,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.slice_anti_entropy.fetches_awaiting_reply.remove(from);
    room_sync::state_machine::retry_fetch(state)
}

// no test_usage necessary
