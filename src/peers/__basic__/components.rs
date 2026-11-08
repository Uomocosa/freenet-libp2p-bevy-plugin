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

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct KeepAlive;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct Room(pub identity::RoomName);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = RoomMembers)]
pub struct InRoom(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
#[relationship_target(relationship = InRoom)]
pub struct RoomMembers(Vec<Entity>);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HistoryHold;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = HeldRooms)]
pub struct HeldBy(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
#[relationship_target(relationship = HeldBy, linked_spawn)]
pub struct HeldRooms(Vec<Entity>);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = HistoryHolders)]
pub struct HoldOf(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
#[relationship_target(relationship = HoldOf, linked_spawn)]
pub struct HistoryHolders(Vec<Entity>);
