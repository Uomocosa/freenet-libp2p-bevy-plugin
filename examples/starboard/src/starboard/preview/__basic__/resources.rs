use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use freenet_libp2p_bevy_plugin::identity;

use crate::starboard;

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct PreviewCursors(pub BTreeMap<identity::PeerId, starboard::room::CursorUpdate>);

// no test_usage necessary
