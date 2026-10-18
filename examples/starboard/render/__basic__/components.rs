use bevy::prelude::Component;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::identity;

#[derive(Component)]
pub struct BoardSprite;

#[derive(Component)]
pub struct BrushOutlineSprite;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RemoteCursorSprite(pub identity::PeerId);
