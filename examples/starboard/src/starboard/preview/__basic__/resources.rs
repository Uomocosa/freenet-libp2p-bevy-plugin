use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::spectator;

use crate::starboard;

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct SlotCursors(
    pub BTreeMap<spectator::SpectateSlot, BTreeMap<identity::PeerId, starboard::room::CursorUpdate>>,
);

// no test_usage necessary
