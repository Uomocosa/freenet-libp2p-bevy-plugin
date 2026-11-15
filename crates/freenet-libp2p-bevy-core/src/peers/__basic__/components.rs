use bevy::prelude::{Component, Entity};
use derive_more::Deref;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Me;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Connected;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct KeepAlive;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = ZoneMembers)]
pub struct InZone(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Deref)]
#[relationship_target(relationship = InZone)]
pub struct ZoneMembers(Vec<Entity>);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HistoryHold;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = HeldZones)]
pub struct HeldBy(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Deref)]
#[relationship_target(relationship = HeldBy, linked_spawn)]
pub struct HeldZones(Vec<Entity>);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
#[relationship(relationship_target = HistoryHolders)]
pub struct HoldOf(pub Entity);

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Deref)]
#[relationship_target(relationship = HoldOf, linked_spawn)]
pub struct HistoryHolders(Vec<Entity>);
