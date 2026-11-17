use bevy::prelude::Component;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::lobby_room::spectator;

#[derive(Component, Deref)]
pub struct SlotCamera(pub spectator::SpectateSlot);

#[derive(Component, Deref)]
pub struct SlotBoardSprite(pub spectator::SpectateSlot);

// no test_usage necessary
