use crate::ui;
use bevy::prelude::*;

use ui::discovery::{CloseStream, StreamedRoom};

pub fn clear_stream_on_close_request(
    mut requests: MessageReader<CloseStream>,
    mut streamed: ResMut<StreamedRoom>,
) {
    if requests.read().next().is_some() {
        **streamed = None;
    }
}
