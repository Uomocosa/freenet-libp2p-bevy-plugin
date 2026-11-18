use bevy::prelude::*;

use crate::p2p;

pub fn send_libp2p_commands(
    mut commands: MessageReader<p2p::Libp2pCommand>,
    sender: Option<Res<p2p::Libp2pCommandSender>>,
) {
    let Some(sender) = sender else {
        commands.clear();
        return;
    };
    for command in commands.read() {
        sender.send(command.clone()).ok();
    }
}
