use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::identity;

#[derive(Resource, Deref)]
pub struct LocalUser(pub identity::Username);
