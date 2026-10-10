#![allow(clippy::needless_pass_by_value)]
use std::ops::Deref;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::net_id;

use crate::args;

#[derive(Resource)]
pub struct Intent {
    pub action: Option<args::Action>,
    pub sent: bool,
}

/// Scripted click: marks the matching button `Interaction::Pressed` so the
/// active UI's own click handler sends the command. Retries every frame until
/// the button exists (a `join:<room>` waits for the room to be discovered).
/// `C` is the UI's create button marker, `J` its per-room button.
pub fn press_once<C, J>(
    mut intent: ResMut<Intent>,
    mut create: Query<&mut Interaction, With<C>>,
    mut join: Query<(&mut Interaction, &J), Without<C>>,
) where
    C: Component,
    J: Component + Deref<Target = net_id::RoomName>,
{
    if intent.sent {
        return;
    }
    let pressed = match &intent.action {
        Some(args::Action::Create) => create
            .iter_mut()
            .next()
            .map(|mut interaction| *interaction = Interaction::Pressed)
            .is_some(),
        Some(args::Action::Join(room)) => join
            .iter_mut()
            .find(|(_, button)| ***button == *room)
            .map(|(mut interaction, _)| *interaction = Interaction::Pressed)
            .is_some(),
        None => true,
    };
    if pressed {
        if let Some(action) = &intent.action {
            tracing::info!("lobby intent pressed {action:?}");
        }
        intent.sent = true;
    }
}

pub fn log_joined(intent: Res<Intent>, mut events: MessageReader<discovery::Event>) {
    for event in events.read() {
        if let discovery::Event::Joined(room) = event {
            match intent.action {
                Some(args::Action::Create) => {
                    tracing::info!("lobby created room={}", room.as_str());
                }
                Some(args::Action::Join(_)) => {
                    tracing::info!("lobby joined room={}", room.as_str());
                }
                None => {}
            }
        }
    }
}
