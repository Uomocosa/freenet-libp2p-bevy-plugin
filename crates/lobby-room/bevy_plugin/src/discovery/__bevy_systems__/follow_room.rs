use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;

pub fn follow_room(
    mut events: MessageReader<discovery::Event>,
    mut commands: MessageWriter<p2p::Libp2pCommand>,
) {
    for event in events.read() {
        match event {
            discovery::Event::Joined(room) => {
                commands.write(p2p::Libp2pCommand::SubscribeZone {
                    zone: identity::ZoneName::clone(room),
                });
            }
            discovery::Event::Left(room) => {
                commands.write(p2p::Libp2pCommand::UnsubscribeZone {
                    zone: identity::ZoneName::clone(room),
                });
            }
            _ => {}
        }
    }
}
