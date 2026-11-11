use std::collections::HashMap;

use bevy::prelude::{Handle, Image, Resource};
use derive_more::{Deref, DerefMut};
use freenet_libp2p_bevy_core::identity;

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct MemberIcons(pub HashMap<identity::PeerId, Handle<Image>>);
