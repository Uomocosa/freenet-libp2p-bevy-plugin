use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::identity;
use crate::presence;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct PeerPresence(pub BTreeMap<identity::PeerId, presence::Whereabouts>);
