use bevy::prelude::{Component, Entity};
use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct Peer(pub identity::PeerId);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Me;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Connected;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct Room(pub identity::RoomName);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = RoomMembers)]
pub struct InRoom(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
#[relationship_target(relationship = InRoom)]
pub struct RoomMembers(Vec<Entity>);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = HistoryHolders)]
pub struct HoldsHistoryOf(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
#[relationship_target(relationship = HoldsHistoryOf)]
pub struct HistoryHolders(Vec<Entity>);
