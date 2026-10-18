use std::collections::BTreeMap;

use bevy::prelude::{Resource, Timer};
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::identity;

use crate::starboard;

#[derive(Resource, Debug, Default, Deref)]
pub struct RoomBrushes(pub BTreeMap<identity::PeerId, starboard::identity::DailyBrush>);

#[derive(Resource, Debug, Deref)]
pub struct BrushRefreshTimer(pub Timer);
