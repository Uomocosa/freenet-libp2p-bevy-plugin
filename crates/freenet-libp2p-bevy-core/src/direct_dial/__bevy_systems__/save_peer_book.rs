use bevy::prelude::*;

use crate::direct_dial;

pub fn save_peer_book(
    mut outputs: MessageReader<direct_dial::state_machine::Output>,
    config: Res<direct_dial::Config>,
) {
    let config = config.into_inner();
    for output in outputs.read() {
        if let direct_dial::state_machine::Output::SavePeerBook(book) = output
            && let Some(path) = config.as_ref()
            && let Err(error) = book.save_to_file(path)
        {
            tracing::error!(target: "direct_dial", "peer book save failed: {error}");
        }
    }
}
